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
    pub prefix: String,
    pub tasks: Vec<Task>,
    pub children: Vec<Project>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub status: String,
    #[serde(default, alias = "issue_type")]
    pub task_type: String,
    #[serde(default)]
    pub priority: i32,
    #[serde(default, alias = "owner")]
    pub assignee: String,
    #[serde(default)]
    pub parent: String,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub updated_at: String,
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

/// Run `bd list --json --tree` and parse the output.
pub fn fetch_tasks() -> Result<Vec<Task>> {
    // Use a child process with a manual timeout via wait_with_output
    let child = Command::new("bd")
        .args(["list", "--json", "--tree"])
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

    let tasks: Vec<Task> =
        serde_json::from_slice(&output.stdout).context("Failed to parse bd JSON output")?;
    Ok(tasks)
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
                || t.id.to_lowercase().contains(query);
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
