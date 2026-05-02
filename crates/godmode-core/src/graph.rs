use anyhow::{Context, Result, bail};
use chrono::Local;
use std::path::{Path, PathBuf};

use crate::integrations::cruxx::{self, EventKind, TaskEvent};
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
/// Emits a cruxx trace event if `root` is provided.
pub fn start(graph: &mut TaskGraph, id: &str) -> Result<()> {
    start_traced(graph, id, None)
}

pub fn start_traced(graph: &mut TaskGraph, id: &str, root: Option<&Path>) -> Result<()> {
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
    if let Some(root) = root {
        let event = TaskEvent {
            kind: EventKind::Started,
            task_id: task.id.clone(),
            title: task.title.clone(),
            crate_name: task.crate_name.clone(),
            commit: None,
            notes: None,
        };
        let _ = cruxx::append_event(root, &event); // non-fatal
    }
    Ok(())
}

/// Mark a running task as done, recording an optional commit SHA.
pub fn complete(
    graph: &mut TaskGraph,
    id: &str,
    commit: Option<&str>,
    notes: Option<&str>,
) -> Result<()> {
    complete_traced(graph, id, commit, notes, None)
}

pub fn complete_traced(
    graph: &mut TaskGraph,
    id: &str,
    commit: Option<&str>,
    notes: Option<&str>,
    root: Option<&Path>,
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
    if let Some(root) = root {
        let event = TaskEvent {
            kind: EventKind::Completed,
            task_id: task.id.clone(),
            title: task.title.clone(),
            crate_name: task.crate_name.clone(),
            commit: task.commit.clone(),
            notes: if task.notes.is_empty() {
                None
            } else {
                Some(task.notes.clone())
            },
        };
        let _ = cruxx::append_event(root, &event); // non-fatal
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

/// Reset all blocked tasks to pending, clearing their notes.
/// Returns the count of tasks unblocked.
pub fn unblock_all(graph: &mut TaskGraph) -> usize {
    let mut count = 0;
    for task in &mut graph.tasks {
        if task.status == Status::Blocked {
            task.status = Status::Pending;
            task.notes = String::new();
            count += 1;
        }
    }
    count
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

/// Remove tasks from the graph. Returns the count removed.
///
/// - `done_only = true`: remove only tasks with `status: done`
/// - `done_only = false`: remove all tasks
pub fn clear(graph: &mut TaskGraph, done_only: bool) -> usize {
    let before = graph.tasks.len();
    if done_only {
        graph.tasks.retain(|t| t.status != Status::Done);
    } else {
        graph.tasks.clear();
    }
    before - graph.tasks.len()
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

    #[test]
    fn clear_done_only_removes_done_tasks() {
        let mut g = TaskGraph::default();
        let mut t1 = Task::new("t1", "A");
        t1.status = Status::Done;
        g.tasks.push(t1);
        g.tasks.push(Task::new("t2", "B")); // pending
        let removed = clear(&mut g, true);
        assert_eq!(removed, 1);
        assert_eq!(g.tasks.len(), 1);
        assert_eq!(g.tasks[0].id, "t2");
    }

    #[test]
    fn unblock_all_resets_blocked_tasks() {
        let mut g = TaskGraph::default();
        let mut t1 = Task::new("t1", "A");
        t1.status = Status::Blocked;
        t1.notes = "reason".into();
        let mut t2 = Task::new("t2", "B");
        t2.status = Status::Done;
        g.tasks.push(t1);
        g.tasks.push(t2);
        let count = unblock_all(&mut g);
        assert_eq!(count, 1);
        assert_eq!(g.tasks[0].status, Status::Pending);
        assert!(g.tasks[0].notes.is_empty());
        assert_eq!(g.tasks[1].status, Status::Done); // unchanged
    }

    #[test]
    fn clear_all_removes_everything() {
        let mut g = TaskGraph::default();
        g.tasks.push(Task::new("t1", "A"));
        g.tasks.push(Task::new("t2", "B"));
        let removed = clear(&mut g, false);
        assert_eq!(removed, 2);
        assert!(g.tasks.is_empty());
    }
}
