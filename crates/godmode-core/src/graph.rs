use anyhow::{Context, Result, bail};
use chrono::Local;
use std::path::{Path, PathBuf};

use crate::model::{Status, Task, TaskGraph};

/// Resolve the task file path relative to a repo root.
pub fn task_file(root: &Path) -> PathBuf {
    root.join(".ctx").join("GODMODE.tasks.yaml")
}

/// Load the task graph from disk. Returns an empty graph if the file does not exist.
pub fn load(root: &Path) -> Result<TaskGraph> {
    let path = task_file(root);
    if !path.exists() {
        return Ok(TaskGraph::default());
    }
    let raw =
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    serde_yaml::from_str(&raw).with_context(|| format!("parsing {}", path.display()))
}

/// Persist the task graph to disk, creating `.ctx/` if needed.
pub fn save(root: &Path, graph: &TaskGraph) -> Result<()> {
    let path = task_file(root);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let raw = serde_yaml::to_string(graph)?;
    std::fs::write(&path, raw).with_context(|| format!("writing {}", path.display()))
}

/// Return all tasks whose dependencies are all `Done` and whose own status is `Pending`.
pub fn runnable(graph: &TaskGraph) -> Vec<&Task> {
    let done_ids: std::collections::HashSet<&str> = graph
        .tasks
        .iter()
        .filter(|t| t.status == Status::Done)
        .map(|t| t.id.as_str())
        .collect();

    graph
        .tasks
        .iter()
        .filter(|t| {
            t.status == Status::Pending
                && t.depends_on
                    .iter()
                    .all(|dep| done_ids.contains(dep.as_str()))
        })
        .collect()
}

/// Mark a task as running. Errors if the task is not pending or has unmet deps.
pub fn start(graph: &mut TaskGraph, id: &str) -> Result<()> {
    let done_ids: std::collections::HashSet<String> = graph
        .tasks
        .iter()
        .filter(|t| t.status == Status::Done)
        .map(|t| t.id.clone())
        .collect();

    let task = graph
        .tasks
        .iter_mut()
        .find(|t| t.id == id)
        .with_context(|| format!("task '{id}' not found"))?;

    if task.status != Status::Pending {
        bail!(
            "task '{}' is {} — can only start pending tasks",
            id,
            task.status
        );
    }
    let blocked: Vec<_> = task
        .depends_on
        .iter()
        .filter(|dep| !done_ids.contains(*dep))
        .collect();
    if !blocked.is_empty() {
        bail!(
            "task '{}' has unmet dependencies: {}",
            id,
            blocked
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    task.status = Status::Running;
    Ok(())
}

/// Mark a running task as done, recording an optional commit SHA.
pub fn complete(
    graph: &mut TaskGraph,
    id: &str,
    commit: Option<&str>,
    notes: Option<&str>,
) -> Result<()> {
    let task = graph
        .tasks
        .iter_mut()
        .find(|t| t.id == id)
        .with_context(|| format!("task '{id}' not found"))?;

    if task.status != Status::Running {
        bail!(
            "task '{}' is {} — can only complete running tasks",
            id,
            task.status
        );
    }
    task.status = Status::Done;
    task.completed = Some(Local::now().date_naive());
    if let Some(sha) = commit {
        task.commit = Some(sha.to_string());
    }
    if let Some(n) = notes
        && !n.is_empty()
    {
        task.notes = n.to_string();
    }
    Ok(())
}

/// Mark a task as blocked with a reason.
pub fn block(graph: &mut TaskGraph, id: &str, reason: &str) -> Result<()> {
    let task = graph
        .tasks
        .iter_mut()
        .find(|t| t.id == id)
        .with_context(|| format!("task '{id}' not found"))?;

    task.status = Status::Blocked;
    task.notes = reason.to_string();
    Ok(())
}

/// Reset a blocked task back to pending.
pub fn unblock(graph: &mut TaskGraph, id: &str) -> Result<()> {
    let task = graph
        .tasks
        .iter_mut()
        .find(|t| t.id == id)
        .with_context(|| format!("task '{id}' not found"))?;

    if task.status != Status::Blocked {
        bail!(
            "task '{}' is {} — can only unblock blocked tasks",
            id,
            task.status
        );
    }
    task.status = Status::Pending;
    task.notes = String::new();
    Ok(())
}

/// Add a new task to the graph.
pub fn add(graph: &mut TaskGraph, task: Task) -> Result<()> {
    if graph.tasks.iter().any(|t| t.id == task.id) {
        bail!("task '{}' already exists", task.id);
    }
    graph.tasks.push(task);
    Ok(())
}

/// Remove a task by id.
pub fn remove(graph: &mut TaskGraph, id: &str) -> Result<()> {
    let before = graph.tasks.len();
    graph.tasks.retain(|t| t.id != id);
    if graph.tasks.len() == before {
        bail!("task '{id}' not found");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Task;

    fn graph_with_chain() -> TaskGraph {
        let mut g = TaskGraph::default();
        g.tasks.push(Task::new("t1", "First"));
        let mut t2 = Task::new("t2", "Second");
        t2.depends_on = vec!["t1".into()];
        g.tasks.push(t2);
        g
    }

    #[test]
    fn runnable_returns_tasks_with_no_deps() {
        let g = graph_with_chain();
        let r = runnable(&g);
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].id, "t1");
    }

    #[test]
    fn runnable_unlocks_after_dep_done() {
        let mut g = graph_with_chain();
        start(&mut g, "t1").unwrap();
        complete(&mut g, "t1", None, None).unwrap();
        let r = runnable(&g);
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].id, "t2");
    }

    #[test]
    fn start_fails_on_unmet_deps() {
        let mut g = graph_with_chain();
        let err = start(&mut g, "t2").unwrap_err();
        assert!(err.to_string().contains("unmet dependencies"));
    }

    #[test]
    fn complete_requires_running_status() {
        let mut g = graph_with_chain();
        let err = complete(&mut g, "t1", None, None).unwrap_err();
        assert!(err.to_string().contains("pending"));
    }

    #[test]
    fn block_sets_status_and_notes() {
        let mut g = graph_with_chain();
        start(&mut g, "t1").unwrap();
        // override to running for block test
        g.tasks[0].status = Status::Running;
        block(&mut g, "t1", "three attempts failed").unwrap();
        assert_eq!(g.tasks[0].status, Status::Blocked);
        assert_eq!(g.tasks[0].notes, "three attempts failed");
    }

    #[test]
    fn add_and_remove_task() {
        let mut g = TaskGraph::default();
        add(&mut g, Task::new("t1", "A")).unwrap();
        assert_eq!(g.tasks.len(), 1);
        remove(&mut g, "t1").unwrap();
        assert!(g.tasks.is_empty());
    }

    #[test]
    fn add_duplicate_fails() {
        let mut g = TaskGraph::default();
        add(&mut g, Task::new("t1", "A")).unwrap();
        let err = add(&mut g, Task::new("t1", "B")).unwrap_err();
        assert!(err.to_string().contains("already exists"));
    }
}
