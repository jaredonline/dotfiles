use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

use crate::data::{self, Project, Task};
use crate::theme;

/// A flattened tree item for rendering and navigation.
#[derive(Clone, Debug)]
pub struct TreeItem {
    pub depth: usize,
    pub kind: TreeItemKind,
    pub expanded: bool,
    pub id: String,
}

#[derive(Clone, Debug)]
pub enum TreeItemKind {
    Project(Project),
    Task(Task),
}

impl TreeItem {
    pub fn display_name(&self) -> String {
        match &self.kind {
            TreeItemKind::Project(p) => p.name.clone(),
            TreeItemKind::Task(t) => t.title.clone(),
        }
    }

    pub fn status(&self) -> Option<&str> {
        match &self.kind {
            TreeItemKind::Project(_) => None,
            TreeItemKind::Task(t) => Some(&t.status),
        }
    }

    pub fn labels(&self) -> &[String] {
        match &self.kind {
            TreeItemKind::Project(p) => &p.labels,
            TreeItemKind::Task(_) => &[],
        }
    }

    pub fn is_collapsible(&self) -> bool {
        match &self.kind {
            TreeItemKind::Project(_) => true,
            TreeItemKind::Task(t) => t.task_type == "epic" || !t.children.is_empty(),
        }
    }
}

/// App state for the UI.
pub struct AppState {
    pub projects: Vec<Project>,
    pub all_labels: Vec<String>,
    pub selected_label: usize, // 0 = ALL
    pub tree_items: Vec<TreeItem>,
    pub selected: usize,
    pub scroll_offset: usize,
    pub filter_text: String,
    pub filter_mode: bool,
    pub bd_error: Option<String>,
    pub collapsed: std::collections::HashSet<String>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            projects: vec![],
            all_labels: vec![],
            selected_label: 0,
            tree_items: vec![],
            selected: 0,
            scroll_offset: 0,
            filter_text: String::new(),
            filter_mode: false,
            bd_error: None,
            collapsed: std::collections::HashSet::new(),
        }
    }

    /// Rebuild the flat tree items from the current projects + filters.
    pub fn rebuild_tree(&mut self) {
        let mut projects = self.projects.clone();

        // Apply label filter
        if self.selected_label > 0 {
            if let Some(label) = self.all_labels.get(self.selected_label - 1) {
                projects = data::filter_by_label(&projects, label);
            }
        }

        // Apply text filter
        if !self.filter_text.is_empty() {
            projects = data::filter_by_text(&projects, &self.filter_text);
        }

        let mut items = Vec::new();
        for project in &projects {
            flatten_project(project, 0, &self.collapsed, &mut items);
        }
        self.tree_items = items;

        // Clamp selection
        if !self.tree_items.is_empty() && self.selected >= self.tree_items.len() {
            self.selected = self.tree_items.len() - 1;
        }
    }

    pub fn toggle_selected(&mut self) {
        if let Some(item) = self.tree_items.get(self.selected) {
            if item.is_collapsible() {
                let id = item.id.clone();
                if self.collapsed.contains(&id) {
                    self.collapsed.remove(&id);
                } else {
                    self.collapsed.insert(id);
                }
                self.rebuild_tree();
            }
        }
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if !self.tree_items.is_empty() && self.selected < self.tree_items.len() - 1 {
            self.selected += 1;
        }
    }

    pub fn cycle_label(&mut self) {
        let total = self.all_labels.len() + 1; // +1 for ALL
        self.selected_label = (self.selected_label + 1) % total;
        self.rebuild_tree();
    }

    pub fn set_status_filter(&mut self, status: &str) {
        if status == "all" {
            self.filter_text.clear();
        } else {
            // Use filter_text as a simple status prefix search
            self.filter_text = status.to_string();
        }
        self.rebuild_tree();
    }

    pub fn selected_task_id(&self) -> Option<&str> {
        self.tree_items.get(self.selected).map(|item| item.id.as_str())
    }
}

fn flatten_project(
    project: &Project,
    depth: usize,
    collapsed: &std::collections::HashSet<String>,
    items: &mut Vec<TreeItem>,
) {
    let id = format!("project:{}", project.id);
    let is_collapsed = collapsed.contains(&id);

    items.push(TreeItem {
        depth,
        kind: TreeItemKind::Project(project.clone()),
        expanded: !is_collapsed,
        id: id.clone(),
    });

    if !is_collapsed {
        for task in &project.tasks {
            flatten_task(task, depth + 1, collapsed, items);
        }
        for child in &project.children {
            flatten_project(child, depth + 1, collapsed, items);
        }
    }
}

fn flatten_task(
    task: &Task,
    depth: usize,
    collapsed: &std::collections::HashSet<String>,
    items: &mut Vec<TreeItem>,
) {
    let id = format!("task:{}", task.id);
    let is_collapsible = task.task_type == "epic" || !task.children.is_empty();
    let is_collapsed = collapsed.contains(&id);

    items.push(TreeItem {
        depth,
        kind: TreeItemKind::Task(task.clone()),
        expanded: !is_collapsed && is_collapsible,
        id: id.clone(),
    });

    if is_collapsible && !is_collapsed {
        for child in &task.children {
            flatten_task(child, depth + 1, collapsed, items);
        }
    }
}

/// Render the full UI.
pub fn render(frame: &mut Frame, state: &AppState) {
    let area = frame.area();

    // Background
    let bg_block = Block::default().style(Style::default().bg(theme::BASE));
    frame.render_widget(bg_block, area);

    let chunks = Layout::vertical([
        Constraint::Length(1), // header
        Constraint::Length(1), // label tabs
        Constraint::Length(1), // metrics bar
        Constraint::Min(3),   // tree
        Constraint::Length(1), // help bar
    ])
    .split(area);

    render_header(frame, chunks[0], state);
    render_label_tabs(frame, chunks[1], state);
    render_metrics(frame, chunks[2], state);
    render_tree(frame, chunks[3], state);
    render_help(frame, chunks[4], state);
}

fn render_header(frame: &mut Frame, area: Rect, state: &AppState) {
    let project_count = data::total_projects(&state.projects);
    let task_count = data::total_tasks(&state.projects);

    let header = Line::from(vec![
        Span::styled(
            " COCKPIT ",
            Style::default()
                .fg(theme::MAUVE)
                .bg(theme::SURFACE0)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  ▸ {} projects  {} tasks ", project_count, task_count),
            Style::default().fg(theme::SUBTEXT0).bg(theme::BASE),
        ),
    ]);

    frame.render_widget(
        Paragraph::new(header).style(Style::default().bg(theme::SURFACE0)),
        area,
    );
}

fn render_label_tabs(frame: &mut Frame, area: Rect, state: &AppState) {
    let mut spans = vec![Span::raw(" ")];

    // ALL tab
    let all_style = if state.selected_label == 0 {
        Style::default()
            .fg(theme::BASE)
            .bg(theme::MAUVE)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme::SUBTEXT0).bg(theme::SURFACE0)
    };
    spans.push(Span::styled(" ALL ", all_style));
    spans.push(Span::raw(" "));

    for (i, label) in state.all_labels.iter().enumerate() {
        let style = if state.selected_label == i + 1 {
            Style::default()
                .fg(theme::BASE)
                .bg(theme::MAUVE)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme::SUBTEXT0).bg(theme::SURFACE0)
        };
        spans.push(Span::styled(format!(" {} ", label.to_uppercase()), style));
        spans.push(Span::raw(" "));
    }

    frame.render_widget(
        Paragraph::new(Line::from(spans)).style(Style::default().bg(theme::BASE)),
        area,
    );
}

fn render_metrics(frame: &mut Frame, area: Rect, state: &AppState) {
    if let Some(err) = &state.bd_error {
        let line = Line::from(vec![
            Span::raw(" "),
            Span::styled(
                format!("⚠ bd unavailable: {}", err),
                Style::default().fg(theme::RED),
            ),
        ]);
        frame.render_widget(
            Paragraph::new(line).style(Style::default().bg(theme::SURFACE0)),
            area,
        );
        return;
    }

    let counts = data::count_by_status(&state.projects);
    let statuses = ["in_progress", "open", "blocked", "deferred", "closed"];

    let mut spans = vec![Span::raw(" ")];
    for status in &statuses {
        let count = counts.get(*status).copied().unwrap_or(0);
        if count > 0 {
            spans.push(Span::styled(
                format!("{} ", theme::status_icon(status)),
                Style::default().fg(theme::status_color(status)),
            ));
            spans.push(Span::styled(
                format!("{} {}  ", count, status),
                Style::default().fg(theme::status_color(status)),
            ));
        }
    }

    frame.render_widget(
        Paragraph::new(Line::from(spans)).style(Style::default().bg(theme::SURFACE0)),
        area,
    );
}

fn render_tree(frame: &mut Frame, area: Rect, state: &AppState) {
    if state.tree_items.is_empty() {
        let msg = if state.bd_error.is_some() {
            "No data available"
        } else if !state.filter_text.is_empty() {
            "No matching tasks"
        } else {
            "No tasks found"
        };
        frame.render_widget(
            Paragraph::new(format!("  {}", msg))
                .style(Style::default().fg(theme::OVERLAY0).bg(theme::BASE)),
            area,
        );
        return;
    }

    let visible_height = area.height as usize;
    let scroll = if state.selected >= visible_height {
        state.selected - visible_height + 1
    } else {
        0
    };

    let mut lines: Vec<Line> = Vec::new();

    for (i, item) in state.tree_items.iter().enumerate().skip(scroll).take(visible_height) {
        let is_selected = i == state.selected;
        let indent = "  ".repeat(item.depth + 1);

        let mut spans = Vec::new();

        // Selection indicator
        if is_selected {
            spans.push(Span::styled("▌", Style::default().fg(theme::MAUVE)));
        } else {
            spans.push(Span::raw(" "));
        }

        spans.push(Span::raw(indent.clone()));

        match &item.kind {
            TreeItemKind::Project(project) => {
                // Collapse indicator
                let arrow = if item.expanded { "▼" } else { "▶" };
                spans.push(Span::styled(
                    format!("{} ", arrow),
                    Style::default().fg(theme::MAUVE),
                ));
                // Project name
                let name_style = if is_selected {
                    Style::default()
                        .fg(theme::TEXT)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme::TEXT)
                };
                spans.push(Span::styled(project.name.clone(), name_style));

                // Labels
                if !project.labels.is_empty() {
                    let label_str = project
                        .labels
                        .iter()
                        .map(|l| l.as_str())
                        .collect::<Vec<_>>()
                        .join(", ");
                    spans.push(Span::styled(
                        format!("  [{}]", label_str),
                        Style::default().fg(theme::OVERLAY0),
                    ));
                }
            }
            TreeItemKind::Task(task) => {
                // For epics, show collapse indicator
                if task.task_type == "epic" || !task.children.is_empty() {
                    let arrow = if item.expanded { "▼" } else { "▶" };
                    spans.push(Span::styled(
                        format!("{} ", arrow),
                        Style::default().fg(theme::PEACH),
                    ));
                    // Epic prefix
                    if task.task_type == "epic" {
                        spans.push(Span::styled(
                            "Epic: ",
                            Style::default()
                                .fg(theme::PEACH)
                                .add_modifier(Modifier::BOLD),
                        ));
                    }
                } else {
                    spans.push(Span::raw("  "));
                }

                // Task title
                let title_style = if is_selected {
                    Style::default()
                        .fg(theme::TEXT)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme::SUBTEXT0)
                };
                spans.push(Span::styled(task.title.clone(), title_style));

                // Status
                let color = theme::status_color(&task.status);
                spans.push(Span::styled(
                    format!("  {} {}", theme::status_icon(&task.status), task.status),
                    Style::default().fg(color),
                ));
            }
        }

        let line_style = if is_selected {
            Style::default().bg(theme::SURFACE0)
        } else {
            Style::default().bg(theme::BASE)
        };

        lines.push(Line::from(spans).style(line_style));
    }

    frame.render_widget(
        Paragraph::new(lines).style(Style::default().bg(theme::BASE)),
        area,
    );

    // Scrollbar
    if state.tree_items.len() > visible_height {
        let mut scrollbar_state = ScrollbarState::new(state.tree_items.len())
            .position(scroll);
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .style(Style::default().fg(theme::SURFACE1)),
            area,
            &mut scrollbar_state,
        );
    }
}

fn render_help(frame: &mut Frame, area: Rect, state: &AppState) {
    let help_text = if state.filter_mode {
        format!("  /{}█  (Enter to apply, Esc to cancel)", state.filter_text)
    } else {
        "  j/k:nav  Enter:expand  Tab:labels  /:filter  1-4:status  r:refresh  q:quit".to_string()
    };

    frame.render_widget(
        Paragraph::new(help_text).style(
            Style::default()
                .fg(theme::OVERLAY0)
                .bg(theme::SURFACE0),
        ),
        area,
    );
}
