use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::process::Command;

#[derive(Deserialize, Clone, Debug)]
pub struct ProjectTree {
    pub projects: Vec<ProjectConfig>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ProjectConfig {
    pub id: String,
    pub name: String,
    #[allow(dead_code)]
    pub path: String,
    pub prefix: String,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub children: Vec<ProjectConfig>,
}

#[derive(Clone, Debug)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub labels: Vec<String>,
    #[allow(dead_code)]
    pub prefix: String,
    pub tasks: Vec<Task>,
    pub children: Vec<Project>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Dependency {
    #[serde(default)]
    pub depends_on_id: String,
    #[serde(default, rename = "type")]
    pub dep_type: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub status: String,
    #[serde(default, alias = "issue_type")]
    pub task_type: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub priority: i32,
    #[serde(default, rename = "owner")]
    #[allow(dead_code)]
    pub assignee: String,
    #[serde(default)]
    pub parent: String,
    #[serde(default)]
    pub dependencies: Vec<Dependency>,
    #[serde(default)]
    #[allow(dead_code)]
    pub labels: Vec<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub updated_at: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub children: Vec<Task>,
}

/// Load project tree from JSON config file.
pub fn load_project_tree(path: &str) -> Result<ProjectTree> {
    let expanded = shellexpand(path);
    let content = std::fs::read_to_string(&expanded)
        .with_context(|| format!("Failed to read project-tree.json at {}", expanded))?;
    let tree: ProjectTree =
        serde_json::from_str(&content).context("Failed to parse project-tree.json")?;
    Ok(tree)
}

/// Run `bd list --json --limit 0` and build tree from parent fields.
pub fn fetch_tasks() -> Result<Vec<Task>> {
    // Use a child process with a manual timeout via wait_with_output
    let child = Command::new("bd")
        .args(["list", "--json", "--limit", "0"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .context("Failed to spawn bd command")?;

    // Wait with a 3-second timeout using a thread
    let (tx, rx) = std::sync::mpsc::channel();
    let handle = std::thread::spawn(move || {
        let result = child.wait_with_output();
        let _ = tx.send(result);
    });

    let output = match rx.recv_timeout(std::time::Duration::from_secs(3)) {
        Ok(result) => result.context("bd command failed")?,
        Err(_) => {
            drop(handle);
            anyhow::bail!("bd command timed out after 3 seconds");
        }
    };

    if !output.status.success() {
        anyhow::bail!(
            "bd exited with status {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let flat_tasks: Vec<Task> =
        serde_json::from_slice(&output.stdout).context("Failed to parse bd JSON output")?;
    Ok(build_task_tree(flat_tasks))
}

/// Resolve the parent ID for a task, checking both the `parent` field
/// and `dependencies` with type "parent" or "parent-child".
fn resolve_parent(task: &Task) -> Option<String> {
    if !task.parent.is_empty() {
        return Some(task.parent.clone());
    }
    task.dependencies
        .iter()
        .find(|d| d.dep_type == "parent" || d.dep_type == "parent-child")
        .map(|d| d.depends_on_id.clone())
}

/// Build a tree of tasks from a flat list using parent fields and dependencies.
pub fn build_task_tree(flat_tasks: Vec<Task>) -> Vec<Task> {
    let ids: std::collections::HashSet<String> =
        flat_tasks.iter().map(|t| t.id.clone()).collect();

    let mut children_of: HashMap<String, Vec<Task>> = HashMap::new();
    let mut roots: Vec<Task> = Vec::new();

    for task in flat_tasks {
        match resolve_parent(&task) {
            Some(parent_id) if ids.contains(&parent_id) => {
                children_of.entry(parent_id).or_default().push(task);
            }
            _ => roots.push(task),
        }
    }

    fn attach_children(task: &mut Task, children_of: &mut HashMap<String, Vec<Task>>) {
        if let Some(mut children) = children_of.remove(&task.id) {
            for child in &mut children {
                attach_children(child, children_of);
            }
            task.children = children;
        }
    }

    for root in &mut roots {
        attach_children(root, &mut children_of);
    }

    roots
}

/// Group tasks into projects based on prefix matching.
pub fn group_tasks(tree: &ProjectTree, tasks: &[Task]) -> Vec<Project> {
    // Build a flat list of all prefixes (including children) sorted by length desc
    // so more specific prefixes match first.
    let mut prefix_map: Vec<(String, Vec<String>)> = Vec::new();
    collect_prefixes(&tree.projects, &[], &mut prefix_map);
    prefix_map.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

    // Assign tasks to projects by prefix
    let mut project_tasks: HashMap<String, Vec<Task>> = HashMap::new();
    let mut uncategorized: Vec<Task> = Vec::new();

    for task in tasks {
        let mut matched = false;
        for (prefix, path) in &prefix_map {
            if task.id.starts_with(prefix) {
                let key = path.join("/");
                project_tasks.entry(key).or_default().push(task.clone());
                matched = true;
                break;
            }
        }
        if !matched {
            uncategorized.push(task.clone());
        }
    }

    let mut projects = build_projects(&tree.projects, &project_tasks);

    if !uncategorized.is_empty() {
        projects.push(Project {
            id: "uncategorized".to_string(),
            name: "Uncategorized".to_string(),
            labels: vec![],
            prefix: "".to_string(),
            tasks: uncategorized,
            children: vec![],
        });
    }

    projects
}

/// Collect all prefixes with their path in the tree.
fn collect_prefixes(
    configs: &[ProjectConfig],
    parent_path: &[String],
    out: &mut Vec<(String, Vec<String>)>,
) {
    for config in configs {
        let mut path = parent_path.to_vec();
        path.push(config.id.clone());
        out.push((config.prefix.clone(), path.clone()));
        collect_prefixes(&config.children, &path, out);
    }
}

/// Build Project structs from configs and grouped tasks.
fn build_projects(
    configs: &[ProjectConfig],
    task_map: &HashMap<String, Vec<Task>>,
) -> Vec<Project> {
    configs
        .iter()
        .map(|config| {
            let key = config.id.clone();
            let tasks = task_map.get(&key).cloned().unwrap_or_default();
            let children = build_projects_with_path(&config.children, task_map, &[config.id.clone()]);
            Project {
                id: config.id.clone(),
                name: config.name.clone(),
                labels: config.labels.clone(),
                prefix: config.prefix.clone(),
                tasks,
                children,
            }
        })
        .collect()
}

fn build_projects_with_path(
    configs: &[ProjectConfig],
    task_map: &HashMap<String, Vec<Task>>,
    parent_path: &[String],
) -> Vec<Project> {
    configs
        .iter()
        .map(|config| {
            let mut path = parent_path.to_vec();
            path.push(config.id.clone());
            let key = path.join("/");
            let tasks = task_map.get(&key).cloned().unwrap_or_default();
            let children = build_projects_with_path(&config.children, task_map, &path);
            Project {
                id: config.id.clone(),
                name: config.name.clone(),
                labels: config.labels.clone(),
                prefix: config.prefix.clone(),
                tasks,
                children,
            }
        })
        .collect()
}

/// Expand ~ in paths.
fn shellexpand(path: &str) -> String {
    if path.starts_with("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return format!("{}{}", home, &path[1..]);
        }
    }
    path.to_string()
}

/// Count tasks by status across all projects.
pub fn count_by_status(projects: &[Project]) -> HashMap<String, usize> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for project in projects {
        count_tasks_recursive(&project.tasks, &mut counts);
        count_by_status_recursive(&project.children, &mut counts);
    }
    counts
}

fn count_by_status_recursive(projects: &[Project], counts: &mut HashMap<String, usize>) {
    for project in projects {
        count_tasks_recursive(&project.tasks, counts);
        count_by_status_recursive(&project.children, counts);
    }
}

fn count_tasks_recursive(tasks: &[Task], counts: &mut HashMap<String, usize>) {
    for task in tasks {
        *counts.entry(task.status.clone()).or_insert(0) += 1;
        count_tasks_recursive(&task.children, counts);
    }
}

/// Count total tasks across all projects.
pub fn total_tasks(projects: &[Project]) -> usize {
    let counts = count_by_status(projects);
    counts.values().sum()
}

/// Count total projects (including children).
pub fn total_projects(projects: &[Project]) -> usize {
    projects
        .iter()
        .map(|p| 1 + total_projects(&p.children))
        .sum()
}

/// Collect all unique labels from projects.
pub fn collect_labels(projects: &[Project]) -> Vec<String> {
    let mut labels: Vec<String> = Vec::new();
    collect_labels_recursive(projects, &mut labels);
    labels.sort();
    labels.dedup();
    labels
}

fn collect_labels_recursive(projects: &[Project], labels: &mut Vec<String>) {
    for project in projects {
        for label in &project.labels {
            if !labels.contains(label) {
                labels.push(label.clone());
            }
        }
        collect_labels_recursive(&project.children, labels);
    }
}

/// Filter projects by label. Returns only projects (and their children) that match.
pub fn filter_by_label(projects: &[Project], label: &str) -> Vec<Project> {
    projects
        .iter()
        .filter_map(|p| {
            if p.labels.iter().any(|l| l == label) {
                Some(p.clone())
            } else {
                let children = filter_by_label(&p.children, label);
                if children.is_empty() {
                    None
                } else {
                    Some(Project {
                        children,
                        tasks: vec![],
                        ..p.clone()
                    })
                }
            }
        })
        .collect()
}

/// Filter projects/tasks by text search (substring match on title/ID).
pub fn filter_by_text(projects: &[Project], query: &str) -> Vec<Project> {
    let query_lower = query.to_lowercase();
    projects
        .iter()
        .filter_map(|p| {
            let matching_tasks: Vec<Task> = filter_tasks_by_text(&p.tasks, &query_lower);
            let matching_children = filter_by_text(&p.children, query);

            if matching_tasks.is_empty() && matching_children.is_empty() {
                None
            } else {
                Some(Project {
                    tasks: matching_tasks,
                    children: matching_children,
                    ..p.clone()
                })
            }
        })
        .collect()
}

fn filter_tasks_by_text(tasks: &[Task], query: &str) -> Vec<Task> {
    tasks
        .iter()
        .filter_map(|t| {
            let matches = t.title.to_lowercase().contains(query)
                || t.id.to_lowercase().contains(query)
                || t.status.to_lowercase().contains(query);
            let matching_children = filter_tasks_by_text(&t.children, query);

            if matches || !matching_children.is_empty() {
                Some(Task {
                    children: if matches {
                        t.children.clone()
                    } else {
                        matching_children
                    },
                    ..t.clone()
                })
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn task(id: &str, title: &str, parent: &str, deps: Vec<(&str, &str)>) -> Task {
        Task {
            id: id.to_string(),
            title: title.to_string(),
            status: "open".to_string(),
            task_type: if title.starts_with("[epic]") { "epic".to_string() } else { "task".to_string() },
            priority: 1,
            assignee: String::new(),
            parent: parent.to_string(),
            dependencies: deps.into_iter().map(|(dep_id, dep_type)| Dependency {
                depends_on_id: dep_id.to_string(),
                dep_type: dep_type.to_string(),
            }).collect(),
            labels: vec![],
            updated_at: String::new(),
            description: String::new(),
            children: vec![],
        }
    }

    fn count_all(tasks: &[Task]) -> usize {
        tasks.iter().map(|t| 1 + count_all(&t.children)).sum()
    }

    fn find_task<'a>(tasks: &'a [Task], id: &str) -> Option<&'a Task> {
        for t in tasks {
            if t.id == id { return Some(t); }
            if let Some(found) = find_task(&t.children, id) { return Some(found); }
        }
        None
    }

    #[test]
    fn test_tree_from_parent_field() {
        let tasks = vec![
            task("epic-1", "[epic] Top", "", vec![]),
            task("task-a", "Child A", "epic-1", vec![]),
            task("task-b", "Child B", "epic-1", vec![]),
        ];
        let tree = build_task_tree(tasks);
        assert_eq!(tree.len(), 1, "should have 1 root");
        assert_eq!(tree[0].children.len(), 2, "epic should have 2 children");
        assert_eq!(count_all(&tree), 3, "total should be 3");
    }

    #[test]
    fn test_tree_from_dependency_type_parent() {
        let tasks = vec![
            task("epic-1", "[epic] Top", "", vec![]),
            task("task-a", "Child A", "", vec![("epic-1", "parent")]),
            task("task-b", "Child B", "", vec![("epic-1", "parent")]),
        ];
        let tree = build_task_tree(tasks);
        assert_eq!(tree.len(), 1, "should have 1 root");
        assert_eq!(tree[0].children.len(), 2, "epic should have 2 children via deps");
    }

    #[test]
    fn test_tree_from_dependency_type_parent_child() {
        let tasks = vec![
            task("epic-1", "[epic] Top", "", vec![]),
            task("task-a", "Child A", "", vec![("epic-1", "parent-child")]),
        ];
        let tree = build_task_tree(tasks);
        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].children.len(), 1);
    }

    #[test]
    fn test_tree_mixed_parent_and_deps() {
        // Some tasks use parent field, others use dependencies
        let tasks = vec![
            task("epic-1", "[epic] Top", "", vec![]),
            task("task-a", "Via parent field", "epic-1", vec![]),
            task("task-b", "Via dep", "", vec![("epic-1", "parent")]),
        ];
        let tree = build_task_tree(tasks);
        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].children.len(), 2);
    }

    #[test]
    fn test_tree_multi_level_nesting() {
        let tasks = vec![
            task("root", "[epic] Root", "", vec![]),
            task("mid", "[epic] Mid", "root", vec![]),
            task("leaf", "Leaf", "mid", vec![]),
        ];
        let tree = build_task_tree(tasks);
        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].children.len(), 1);
        assert_eq!(tree[0].children[0].children.len(), 1);
        assert_eq!(tree[0].children[0].children[0].id, "leaf");
    }

    #[test]
    fn test_tree_deep_nesting_via_deps() {
        let tasks = vec![
            task("l0", "[epic] Level 0", "", vec![]),
            task("l1", "[epic] Level 1", "", vec![("l0", "parent")]),
            task("l2", "[epic] Level 2", "", vec![("l1", "parent")]),
            task("l3", "Level 3", "", vec![("l2", "parent")]),
        ];
        let tree = build_task_tree(tasks);
        assert_eq!(tree.len(), 1);
        assert_eq!(count_all(&tree), 4);
        let l3 = find_task(&tree, "l3").expect("should find l3");
        assert_eq!(l3.title, "Level 3");
    }

    #[test]
    fn test_tree_orphan_parent_stays_root() {
        // Task references a parent that doesn't exist in the list
        let tasks = vec![
            task("task-a", "Orphan", "nonexistent", vec![]),
            task("task-b", "Root", "", vec![]),
        ];
        let tree = build_task_tree(tasks);
        assert_eq!(tree.len(), 2, "orphan should be treated as root");
    }

    #[test]
    fn test_tree_preserves_all_tasks() {
        let tasks = vec![
            task("epic-1", "[epic] Epic 1", "", vec![]),
            task("epic-2", "[epic] Epic 2", "", vec![]),
            task("t1", "Task 1", "epic-1", vec![]),
            task("t2", "Task 2", "", vec![("epic-1", "parent")]),
            task("t3", "Task 3", "epic-2", vec![]),
            task("t4", "Task 4", "", vec![]),
            task("t5", "Orphan", "gone", vec![]),
        ];
        let tree = build_task_tree(tasks);
        assert_eq!(count_all(&tree), 7, "all tasks must be accounted for");
    }

    #[test]
    fn test_parent_field_takes_precedence_over_dep() {
        // Task has both parent field and a different dep — parent field wins
        let tasks = vec![
            task("epic-a", "[epic] A", "", vec![]),
            task("epic-b", "[epic] B", "", vec![]),
            task("t1", "Task", "epic-a", vec![("epic-b", "parent")]),
        ];
        let tree = build_task_tree(tasks);
        let epic_a = find_task(&tree, "epic-a").unwrap();
        assert_eq!(epic_a.children.len(), 1, "parent field should win");
        let epic_b = find_task(&tree, "epic-b").unwrap();
        assert_eq!(epic_b.children.len(), 0);
    }
}
