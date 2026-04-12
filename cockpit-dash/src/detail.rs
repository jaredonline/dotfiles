use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
    Frame,
};

use crate::data::{Project, Task};
use crate::theme;
use crate::ui::AppState;

fn find_in_tasks<'a>(tasks: &'a [Task], id: &str) -> Option<&'a Task> {
    for task in tasks {
        if task.id == id {
            return Some(task);
        }
        if let Some(found) = find_in_tasks(&task.children, id) {
            return Some(found);
        }
    }
    None
}

pub fn find_task_by_id<'a>(projects: &'a [Project], id: &str) -> Option<&'a Task> {
    for project in projects {
        if let Some(found) = find_in_tasks(&project.tasks, id) {
            return Some(found);
        }
        if let Some(found) = find_task_by_id(&project.children, id) {
            return Some(found);
        }
    }
    None
}

pub fn render_detail(frame: &mut Frame, area: Rect, state: &AppState) -> u16 {
    let chunks = Layout::vertical([
        Constraint::Length(1), // breadcrumb bar
        Constraint::Min(3),   // scrollable content
        Constraint::Length(1), // help bar
    ])
    .split(area);

    // Render breadcrumb
    render_breadcrumb(frame, chunks[0], state);

    // Render help bar
    render_help(frame, chunks[2]);

    // Resolve current task
    let current_id = match state.current_detail_id() {
        Some(id) => id,
        None => {
            let msg = Paragraph::new("Task not found")
                .alignment(ratatui::layout::Alignment::Center)
                .style(Style::default().fg(theme::OVERLAY0).bg(theme::BASE));
            frame.render_widget(msg, chunks[1]);
            return 0;
        }
    };

    let task = match find_task_by_id(&state.projects, current_id) {
        Some(t) => t,
        None => {
            let msg = Paragraph::new("Task not found")
                .alignment(ratatui::layout::Alignment::Center)
                .style(Style::default().fg(theme::OVERLAY0).bg(theme::BASE));
            frame.render_widget(msg, chunks[1]);
            return 0;
        }
    };

    // Build content lines
    let mut lines: Vec<Line> = Vec::new();

    // Metadata block: title (bold)
    lines.push(Line::from(Span::styled(
        task.title.clone(),
        Style::default()
            .fg(theme::TEXT)
            .add_modifier(Modifier::BOLD),
    )));

    // Status / priority / owner
    let status_color = theme::status_color(&task.status);
    let mut meta_spans = vec![
        Span::styled(
            format!("{} {}", theme::status_icon(&task.status), task.status),
            Style::default().fg(status_color),
        ),
        Span::styled("  priority: ", Style::default().fg(theme::OVERLAY0)),
        Span::styled(
            format!("{}", task.priority),
            Style::default().fg(theme::TEXT),
        ),
    ];
    if !task.assignee.is_empty() {
        meta_spans.push(Span::styled("  owner: ", Style::default().fg(theme::OVERLAY0)));
        meta_spans.push(Span::styled(
            task.assignee.clone(),
            Style::default().fg(theme::TEXT),
        ));
    }
    lines.push(Line::from(meta_spans));

    // ID / updated_at
    let mut id_spans = vec![
        Span::styled("ID: ", Style::default().fg(theme::OVERLAY0)),
        Span::styled(task.id.clone(), Style::default().fg(theme::TEXT)),
    ];
    if !task.updated_at.is_empty() {
        id_spans.push(Span::styled("  updated: ", Style::default().fg(theme::OVERLAY0)));
        id_spans.push(Span::styled(
            task.updated_at.clone(),
            Style::default().fg(theme::TEXT),
        ));
    }
    lines.push(Line::from(id_spans));

    // Separator
    lines.push(Line::from(Span::styled(
        "─".repeat(area.width as usize),
        Style::default().fg(theme::SURFACE1),
    )));

    // Description
    if !task.description.is_empty() {
        for desc_line in task.description.split('\n') {
            lines.push(Line::from(Span::styled(
                desc_line.to_string(),
                Style::default().fg(theme::SUBTEXT0),
            )));
        }
    }

    // Children section
    if !task.children.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("Children ({}):", task.children.len()),
            Style::default()
                .fg(theme::TEXT)
                .add_modifier(Modifier::BOLD),
        )));
        for (i, child) in task.children.iter().enumerate() {
            let child_status_color = theme::status_color(&child.status);
            lines.push(Line::from(vec![
                Span::styled(
                    format!("[{}] ", i + 1),
                    Style::default().fg(theme::MAUVE),
                ),
                Span::styled(child.title.clone(), Style::default().fg(theme::TEXT)),
                Span::styled("  ", Style::default()),
                Span::styled(
                    format!("{} {}", theme::status_icon(&child.status), child.status),
                    Style::default().fg(child_status_color),
                ),
            ]));
        }
    }

    let content_height = lines.len() as u16;

    // Render scrollable content
    let content = Paragraph::new(lines)
        .scroll((state.detail_scroll, 0))
        .wrap(Wrap { trim: false })
        .style(Style::default().bg(theme::BASE));
    frame.render_widget(content, chunks[1]);

    // Scrollbar
    let visible_height = chunks[1].height as u16;
    if content_height > visible_height {
        let mut scrollbar_state = ScrollbarState::new(content_height as usize)
            .position(state.detail_scroll as usize);
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .style(Style::default().fg(theme::SURFACE1)),
            chunks[1],
            &mut scrollbar_state,
        );
    }

    content_height
}

fn render_breadcrumb(frame: &mut Frame, area: Rect, state: &AppState) {
    let mut spans = vec![
        Span::styled("< List", Style::default().fg(theme::MAUVE)),
    ];

    for (i, id) in state.nav_stack.iter().enumerate() {
        spans.push(Span::styled(" > ", Style::default().fg(theme::OVERLAY0)));
        let is_last = i == state.nav_stack.len() - 1;
        let title = find_task_by_id(&state.projects, id)
            .map(|t| t.title.clone())
            .unwrap_or_else(|| id.clone());
        if is_last {
            spans.push(Span::styled(
                title,
                Style::default()
                    .fg(theme::TEXT)
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            spans.push(Span::styled(title, Style::default().fg(theme::SUBTEXT0)));
        }
    }

    frame.render_widget(
        Paragraph::new(Line::from(spans)).style(
            Style::default()
                .fg(theme::TEXT)
                .bg(theme::SURFACE0),
        ),
        area,
    );
}

fn render_help(frame: &mut Frame, area: Rect) {
    let help_text = "  j/k:scroll  G:end  gg:top  PgUp/Dn  1-9:children  h/<-:back  q:quit";
    frame.render_widget(
        Paragraph::new(help_text).style(
            Style::default()
                .fg(theme::OVERLAY0)
                .bg(theme::SURFACE0),
        ),
        area,
    );
}

pub fn handle_detail_key(
    key: KeyEvent,
    state: &mut AppState,
) -> anyhow::Result<bool> {
    // Check pending_g FIRST
    if state.pending_g {
        if key.code == KeyCode::Char('g') {
            state.detail_scroll = 0;
            state.pending_g = false;
            return Ok(false);
        } else {
            state.pending_g = false;
            // Fall through to normal dispatch
        }
    }

    match key.code {
        KeyCode::Char('q') => return Ok(true),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Ok(true),

        KeyCode::Char('j') | KeyCode::Down => {
            state.detail_scroll = state
                .detail_scroll
                .saturating_add(1)
                .min(state.last_content_height.saturating_sub(20));
        }
        KeyCode::Char('k') | KeyCode::Up => {
            state.detail_scroll = state.detail_scroll.saturating_sub(1);
        }

        KeyCode::Char('G') | KeyCode::End => {
            state.detail_scroll = state.last_content_height.saturating_sub(20);
        }
        KeyCode::Home => {
            state.detail_scroll = 0;
        }

        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.detail_scroll = state
                .detail_scroll
                .saturating_add(10)
                .min(state.last_content_height.saturating_sub(20));
        }
        KeyCode::PageDown => {
            state.detail_scroll = state
                .detail_scroll
                .saturating_add(10)
                .min(state.last_content_height.saturating_sub(20));
        }
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.detail_scroll = state.detail_scroll.saturating_sub(10);
        }
        KeyCode::PageUp => {
            state.detail_scroll = state.detail_scroll.saturating_sub(10);
        }

        KeyCode::Left | KeyCode::Char('h') | KeyCode::Esc => {
            state.exit_detail();
        }

        KeyCode::Char('o') => {
            if let Some(id) = state.current_detail_id() {
                let id = id.to_string();
                let _ = std::process::Command::new("bd")
                    .args(["edit", &id])
                    .spawn();
            }
        }

        KeyCode::Char(c @ '1'..='9') => {
            let n = (c as usize) - ('0' as usize);
            if let Some(current_id) = state.current_detail_id() {
                let current_id = current_id.to_string();
                if let Some(task) = find_task_by_id(&state.projects, &current_id) {
                    if let Some(child) = task.children.get(n - 1) {
                        let child_id = child.id.clone();
                        state.enter_detail(child_id);
                    }
                }
            }
        }

        KeyCode::Char('g') => {
            state.pending_g = true;
        }

        _ => {}
    }

    Ok(false)
}
