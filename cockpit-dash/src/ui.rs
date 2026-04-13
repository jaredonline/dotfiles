use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

use crate::data::{self, Project, Task};
use crate::theme;

#[derive(Clone, Debug, PartialEq)]
pub enum ViewMode {
    List,
    Detail,
    Filter,
}

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
    LabelGroup {
        #[allow(dead_code)]
        key: String,
        name: String,
    },
    Task(Task),
}

impl TreeItem {
    #[allow(dead_code)]
    pub fn display_name(&self) -> String {
        match &self.kind {
            TreeItemKind::Project(p) => p.name.clone(),
            TreeItemKind::LabelGroup { name, .. } => name.clone(),
            TreeItemKind::Task(t) => t.title.clone(),
        }
    }

    #[allow(dead_code)]
    pub fn status(&self) -> Option<&str> {
        match &self.kind {
            TreeItemKind::Project(_) => None,
            TreeItemKind::LabelGroup { .. } => None,
            TreeItemKind::Task(t) => Some(&t.status),
        }
    }

    #[allow(dead_code)]
    pub fn labels(&self) -> &[String] {
        match &self.kind {
            TreeItemKind::Project(_) => &[],
            TreeItemKind::LabelGroup { .. } => &[],
            TreeItemKind::Task(_) => &[],
        }
    }

    pub fn is_collapsible(&self) -> bool {
        match &self.kind {
            TreeItemKind::Project(_) => true,
            TreeItemKind::LabelGroup { .. } => true,
            TreeItemKind::Task(t) => t.task_type == "epic" || !t.children.is_empty(),
        }
    }
}

/// App state for the UI.
pub struct AppState {
    pub projects: Vec<Project>,
    pub selected_project: usize, // 0 = ALL
    pub tree_items: Vec<TreeItem>,
    pub selected: usize,
    #[allow(dead_code)]
    pub scroll_offset: usize,
    pub filter_text: String,
    pub bd_error: Option<String>,
    pub collapsed: std::collections::HashSet<String>,
    pub filtered_projects: Vec<Project>,
    pub view_mode: ViewMode,
    pub nav_stack: Vec<String>,
    pub detail_scroll: u16,
    pub pending_g: bool,
    pub last_content_height: u16,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            projects: vec![],
            selected_project: 0,
            tree_items: vec![],
            selected: 0,
            scroll_offset: 0,
            filter_text: String::new(),
            bd_error: None,
            collapsed: std::collections::HashSet::new(),
            filtered_projects: vec![],
            view_mode: ViewMode::List,
            nav_stack: vec![],
            detail_scroll: 0,
            pending_g: false,
            last_content_height: 0,
        }
    }

    /// Rebuild the flat tree items from the current projects + filters.
    pub fn rebuild_tree(&mut self) {
        let mut items = Vec::new();

        if self.selected_project == 0 {
            // ALL view: projects -> tasks (no label sub-grouping)
            let mut projects = self.projects.clone();
            if !self.filter_text.is_empty() {
                projects = data::filter_by_text(&projects, &self.filter_text);
            }
            self.filtered_projects = projects;
            for project in &self.filtered_projects {
                flatten_project(project, 0, &self.collapsed, &mut items);
            }
        } else if let Some(project) = self.projects.get(self.selected_project - 1) {
            // Project view: label groups -> tasks
            self.filtered_projects = vec![project.clone()];

            // Collect all tasks from project + children recursively
            let all_tasks = collect_all_tasks(project);

            // Apply text filter
            let tasks: Vec<&Task> = if self.filter_text.is_empty() {
                all_tasks.iter().collect()
            } else {
                let query = self.filter_text.to_lowercase();
                all_tasks.iter().filter(|t| {
                    t.title.to_lowercase().contains(&query)
                        || t.id.to_lowercase().contains(&query)
                        || t.status.to_lowercase().contains(&query)
                }).collect()
            };

            let mut matched_ids = std::collections::HashSet::new();

            for label_def in &project.label_defs {
                let matching: Vec<&Task> = tasks.iter()
                    .filter(|t| t.labels.iter().any(|l| l == &label_def.key))
                    .copied()
                    .collect();

                if !matching.is_empty() {
                    let group_id = format!("label:{}", label_def.key);
                    let is_collapsed = self.collapsed.contains(&group_id);

                    items.push(TreeItem {
                        depth: 0,
                        kind: TreeItemKind::LabelGroup {
                            key: label_def.key.clone(),
                            name: label_def.name.clone(),
                        },
                        expanded: !is_collapsed,
                        id: group_id,
                    });

                    if !is_collapsed {
                        for task in &matching {
                            matched_ids.insert(task.id.clone());
                            flatten_task(task, 1, &self.collapsed, &mut items);
                        }
                    }
                }
            }

            // "Other" group for unmatched tasks
            let other: Vec<&Task> = tasks.iter()
                .filter(|t| !matched_ids.contains(&t.id))
                .copied()
                .collect();

            if !other.is_empty() {
                let group_id = "label:_other".to_string();
                let is_collapsed = self.collapsed.contains(&group_id);

                items.push(TreeItem {
                    depth: 0,
                    kind: TreeItemKind::LabelGroup {
                        key: "_other".to_string(),
                        name: "Other".to_string(),
                    },
                    expanded: !is_collapsed,
                    id: group_id,
                });

                if !is_collapsed {
                    for task in &other {
                        flatten_task(task, 1, &self.collapsed, &mut items);
                    }
                }
            }
        }

        self.tree_items = items;
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

    pub fn cycle_project(&mut self) {
        let total = self.projects.len() + 1; // +1 for ALL
        self.selected_project = (self.selected_project + 1) % total;
        self.rebuild_tree();
    }

    pub fn cycle_project_backward(&mut self) {
        let total = self.projects.len() + 1;
        self.selected_project = (self.selected_project + total - 1) % total;
        self.rebuild_tree();
    }

    #[allow(dead_code)]
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

    /// Enter detail view for the given task ID. Pushes to nav stack.
    pub fn enter_detail(&mut self, task_id: String) {
        self.nav_stack.push(task_id);
        self.detail_scroll = 0;
        self.pending_g = false;
        self.view_mode = ViewMode::Detail;
    }

    /// Go back from detail view. Pops nav stack.
    pub fn exit_detail(&mut self) {
        self.nav_stack.pop();
        if self.nav_stack.is_empty() {
            self.view_mode = ViewMode::List;
        } else {
            self.detail_scroll = 0;
            self.pending_g = false;
        }
    }

    /// Get the current detail task ID (last item on nav stack).
    pub fn current_detail_id(&self) -> Option<&str> {
        self.nav_stack.last().map(|s| s.as_str())
    }
}

fn collect_all_tasks(project: &Project) -> Vec<Task> {
    let mut tasks = Vec::new();
    fn collect(tasks_list: &[Task], out: &mut Vec<Task>) {
        for t in tasks_list {
            out.push(t.clone());
            collect(&t.children, out);
        }
    }
    collect(&project.tasks, &mut tasks);
    for child in &project.children {
        tasks.extend(collect_all_tasks(child));
    }
    tasks
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

/// Render the full UI. Returns content height for detail view scroll bounds.
pub fn render(frame: &mut Frame, state: &AppState) -> u16 {
    let area = frame.area();

    // Background
    let bg_block = Block::default().style(Style::default().bg(theme::BASE));
    frame.render_widget(bg_block, area);

    match state.view_mode {
        ViewMode::List | ViewMode::Filter => {
            let ip_items = collect_in_progress(&state.filtered_projects);

            let chunks = Layout::vertical([
                Constraint::Length(1), // header
                Constraint::Length(1), // label tabs
                Constraint::Length(1), // metrics bar
                Constraint::Min(3),   // content (tree + optional side panel)
                Constraint::Length(1), // help bar
            ])
            .split(area);

            render_header(frame, chunks[0], state);
            render_project_tabs(frame, chunks[1], state);
            render_metrics(frame, chunks[2], state);

            if ip_items.is_empty() {
                render_tree(frame, chunks[3], state);
            } else {
                let content = Layout::horizontal([
                    Constraint::Min(40),
                    Constraint::Length(40),
                ])
                .split(chunks[3]);

                render_tree(frame, content[0], state);
                render_in_progress(frame, content[1], &ip_items);
            }

            render_help(frame, chunks[4], state);
            0
        }
        ViewMode::Detail => {
            crate::detail::render_detail(frame, area, state)
        }
    }
}

fn collect_in_progress(projects: &[Project]) -> Vec<(String, String)> {
    let mut result = Vec::new();
    for project in projects {
        collect_ip_from_project(project, &mut result);
    }
    result
}

fn collect_ip_from_project(project: &Project, result: &mut Vec<(String, String)>) {
    collect_ip_from_tasks(&project.tasks, &project.name, result);
    for child in &project.children {
        collect_ip_from_project(child, result);
    }
}

fn collect_ip_from_tasks(tasks: &[Task], project_name: &str, result: &mut Vec<(String, String)>) {
    for task in tasks {
        if task.status == "in_progress" {
            result.push((project_name.to_string(), task.title.clone()));
        }
        collect_ip_from_tasks(&task.children, project_name, result);
    }
}

fn render_in_progress(frame: &mut Frame, area: Rect, items: &[(String, String)]) {
    let title = Line::from(vec![
        Span::styled(" In Progress ", Style::default()
            .fg(theme::GREEN).add_modifier(Modifier::BOLD)),
        Span::styled(format!("({})", items.len()), Style::default().fg(theme::OVERLAY0)),
        Span::raw(" "),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::SURFACE1))
        .title(title)
        .style(Style::default().bg(theme::SURFACE0));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let max_tasks = inner.height as usize;
    let has_overflow = items.len() > max_tasks;
    let show_count = if has_overflow { max_tasks.saturating_sub(1) } else { max_tasks };

    let mut lines = Vec::new();
    for (project_name, task_title) in items.iter().take(show_count) {
        lines.push(Line::from(vec![
            Span::styled("● ", Style::default().fg(theme::GREEN)),
            Span::styled(task_title.as_str(), Style::default().fg(theme::TEXT)),
        ]));
        lines.push(Line::from(Span::styled(
            format!("  {}", project_name),
            Style::default().fg(theme::OVERLAY0),
        )));
    }

    if has_overflow {
        lines.push(Line::from(Span::styled(
            format!("  +{} more", items.len() - show_count),
            Style::default().fg(theme::OVERLAY0),
        )));
    }

    frame.render_widget(
        Paragraph::new(lines),
        inner,
    );
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

fn render_project_tabs(frame: &mut Frame, area: Rect, state: &AppState) {
    let mut spans = vec![Span::raw(" ")];

    // ALL tab
    let all_style = if state.selected_project == 0 {
        Style::default()
            .fg(theme::BASE)
            .bg(theme::MAUVE)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme::SUBTEXT0).bg(theme::SURFACE0)
    };
    spans.push(Span::styled(" ALL ", all_style));
    spans.push(Span::raw(" "));

    for (i, project) in state.projects.iter().enumerate() {
        let style = if state.selected_project == i + 1 {
            Style::default()
                .fg(theme::BASE)
                .bg(theme::MAUVE)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme::SUBTEXT0).bg(theme::SURFACE0)
        };
        spans.push(Span::styled(format!(" {} ", project.name), style));
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
                if !project.label_defs.is_empty() {
                    let label_str = project
                        .label_defs
                        .iter()
                        .map(|l| l.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ");
                    spans.push(Span::styled(
                        format!("  [{}]", label_str),
                        Style::default().fg(theme::OVERLAY0),
                    ));
                }
            }
            TreeItemKind::LabelGroup { name, .. } => {
                let arrow = if item.expanded { "▼" } else { "▶" };
                spans.push(Span::styled(
                    format!("{} ", arrow),
                    Style::default().fg(theme::SAPPHIRE),
                ));
                let name_style = if is_selected {
                    Style::default()
                        .fg(theme::TEXT)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme::TEXT)
                };
                spans.push(Span::styled(name.clone(), name_style));
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

                // Beads ID
                spans.push(Span::styled(
                    format!("  ({})", task.id),
                    Style::default().fg(theme::OVERLAY0),
                ));

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
    let help_text = if state.view_mode == ViewMode::Filter {
        format!("  /{}█  (Enter to apply, Esc to cancel)", state.filter_text)
    } else {
        "  j/k:nav  Enter:expand  Tab/S-Tab:projects  /:filter  1-4:status  r:refresh  q:quit".to_string()
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
