mod data;
mod theme;
mod ui;

use anyhow::{Context, Result};
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::*;
use std::io::stdout;
use std::time::{Duration, Instant};

use ui::AppState;

#[derive(Parser)]
#[command(name = "cockpit-dash", about = "Terminal dashboard for beads tasks")]
struct Cli {
    /// Path to project-tree.yml
    #[arg(long, default_value_t = default_config_path())]
    config: String,

    /// Refresh interval in seconds
    #[arg(long, default_value_t = 5)]
    refresh: u64,

    /// Filter to projects matching label
    #[arg(long)]
    label: Option<String>,

    /// Filter to specific project ID
    #[arg(long)]
    project: Option<String>,
}

fn default_config_path() -> String {
    if let Ok(cockpit_dir) = std::env::var("COCKPIT_DIR") {
        format!("{}/project-tree.yml", cockpit_dir)
    } else if let Ok(home) = std::env::var("HOME") {
        format!("{}/ai-cockpit/project-tree.yml", home)
    } else {
        "project-tree.yml".to_string()
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load project tree config
    let project_tree = data::load_project_tree(&cli.config)?;

    // Initialize app state
    let mut state = AppState::new();
    state.all_labels = data::collect_labels(&vec![]);

    // Apply initial label filter from CLI
    if let Some(label) = &cli.label {
        // Find the label index
        let labels = data::collect_labels(&vec![]);
        if let Some(idx) = labels.iter().position(|l| l == label) {
            state.selected_label = idx + 1;
        }
    }

    // Initial data load
    refresh_data(&project_tree, &mut state, cli.project.as_deref());

    // Setup terminal
    enable_raw_mode().context("Failed to enable raw mode")?;
    stdout()
        .execute(EnterAlternateScreen)
        .context("Failed to enter alternate screen")?;

    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend).context("Failed to create terminal")?;

    let result = run_app(&mut terminal, &mut state, &project_tree, &cli);

    // Restore terminal
    disable_raw_mode().context("Failed to disable raw mode")?;
    stdout()
        .execute(LeaveAlternateScreen)
        .context("Failed to leave alternate screen")?;

    result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    state: &mut AppState,
    project_tree: &data::ProjectTree,
    cli: &Cli,
) -> Result<()> {
    let refresh_interval = Duration::from_secs(cli.refresh);
    let mut last_refresh = Instant::now();

    loop {
        // Render
        terminal.draw(|frame| ui::render(frame, state))?;

        // Poll for events with timeout for refresh
        let timeout = refresh_interval
            .checked_sub(last_refresh.elapsed())
            .unwrap_or(Duration::ZERO);

        if event::poll(timeout).context("Event poll failed")? {
            if let Event::Key(key) = event::read().context("Event read failed")? {
                if handle_key(key, state, project_tree, cli)? {
                    return Ok(());
                }
            }
        }

        // Timer-based refresh
        if last_refresh.elapsed() >= refresh_interval {
            refresh_data(project_tree, state, cli.project.as_deref());
            last_refresh = Instant::now();
        }
    }
}

fn handle_key(
    key: KeyEvent,
    state: &mut AppState,
    project_tree: &data::ProjectTree,
    cli: &Cli,
) -> Result<bool> {
    // Filter mode handles input differently
    if state.filter_mode {
        match key.code {
            KeyCode::Esc => {
                state.filter_mode = false;
                state.filter_text.clear();
                state.rebuild_tree();
            }
            KeyCode::Enter => {
                state.filter_mode = false;
                state.rebuild_tree();
            }
            KeyCode::Backspace => {
                state.filter_text.pop();
                state.rebuild_tree();
            }
            KeyCode::Char(c) => {
                state.filter_text.push(c);
                state.rebuild_tree();
            }
            _ => {}
        }
        return Ok(false);
    }

    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => return Ok(true),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Ok(true),

        // Navigation
        KeyCode::Char('j') | KeyCode::Down => state.move_down(),
        KeyCode::Char('k') | KeyCode::Up => state.move_up(),

        // Expand/collapse
        KeyCode::Enter => state.toggle_selected(),

        // Label cycling
        KeyCode::Tab => state.cycle_label(),

        // Text filter
        KeyCode::Char('/') => {
            state.filter_mode = true;
            state.filter_text.clear();
        }

        // Quick status filters
        KeyCode::Char('1') => {
            state.filter_text.clear();
            state.rebuild_tree();
        }
        KeyCode::Char('2') => {
            state.filter_text = "in_progress".to_string();
            state.rebuild_tree();
        }
        KeyCode::Char('3') => {
            state.filter_text = "open".to_string();
            state.rebuild_tree();
        }
        KeyCode::Char('4') => {
            state.filter_text = "blocked".to_string();
            state.rebuild_tree();
        }

        // Force refresh
        KeyCode::Char('r') => {
            refresh_data(project_tree, state, cli.project.as_deref());
        }

        // Open in editor
        KeyCode::Char('o') => {
            if let Some(task_id) = state.selected_task_id() {
                let task_id = task_id.to_string();
                // Strip prefix from task ID
                if let Some(id) = task_id.strip_prefix("task:") {
                    open_in_editor(id);
                }
            }
        }

        // Quick-add todo
        KeyCode::Char('n') => {
            // Exit TUI temporarily to get input
            // For now, this is a no-op since we'd need to handle terminal state
            // TODO: implement inline input
        }

        _ => {}
    }

    Ok(false)
}

fn refresh_data(
    project_tree: &data::ProjectTree,
    state: &mut AppState,
    project_filter: Option<&str>,
) {
    match data::fetch_tasks() {
        Ok(tasks) => {
            state.bd_error = None;
            let mut projects = data::group_tasks(project_tree, &tasks);

            // Apply project filter from CLI
            if let Some(proj_id) = project_filter {
                projects.retain(|p| p.id == proj_id);
            }

            state.projects = projects;
            state.all_labels = data::collect_labels(&state.projects);
            state.rebuild_tree();
        }
        Err(e) => {
            state.bd_error = Some(format!("{:#}", e));
        }
    }
}

fn open_in_editor(task_id: &str) {
    // Try to open via bd edit
    if let Ok(output) = std::process::Command::new("bd")
        .args(["show", task_id, "--json"])
        .output()
    {
        if output.status.success() {
            // For now, just try to open bd edit
            let _ = std::process::Command::new("bd")
                .args(["edit", task_id])
                .status();
        }
    }
}
