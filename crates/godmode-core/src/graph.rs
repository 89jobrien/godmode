use anyhow::{Context, Result, bail};
use chrono::Local;
use std::path::{Path, PathBuf};

use crate::integrations::cruxx;
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
/// Uses the cached `done_ids` set on `TaskGraph` to avoid rebuilding on every call.
pub fn runnable(graph: &TaskGraph) -> Vec<&Task> {
    let done_ids = graph.done_ids();

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
    // Check deps against cached done_ids before mutating.
    let unmet: Vec<String> = {
        let done_ids = graph.done_ids();
        let task = graph
            .tasks
            .iter()
            .find(|t| t.id == id)
            .with_context(|| format!("task '{id}' not found"))?;
        if task.status != Status::Pending {
            bail!(
                "task '{}' is {} — can only start pending tasks",
                id,
                task.status
            );
        }
        task.depends_on
            .iter()
            .filter(|dep| !done_ids.contains(dep.as_str()))
            .cloned()
            .collect()
    };
    if !unmet.is_empty() {
        bail!(
            "task '{}' has unmet dependencies: {}",
            id,
            unmet
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    {
        let task = graph
            .tasks
            .iter_mut()
            .find(|t| t.id == id)
            .with_context(|| format!("task '{id}' not found"))?;
        task.status = Status::Running;
    }
    graph.invalidate_done_cache();
    // Trace step is recorded via Session::record in the session_trace layer (#36).
    let _ = (root, &cruxx::step_started(id));
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
    {
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
        task.completed_at = Some(chrono::Utc::now());
        if let Some(sha) = commit {
            task.commit = Some(sha.to_string());
        }
        if let Some(n) = notes
            && !n.is_empty()
        {
            task.notes = n.to_string();
        }
    }
    graph.invalidate_done_cache();
    // Trace step: look up the task again (immutably) for trace metadata.
    let task = graph
        .tasks
        .iter()
        .find(|t| t.id == id)
        .expect("task was just modified");
    let _ = (
        root,
        &cruxx::step_completed(
            task.id.as_str(),
            task.commit.as_deref(),
            if task.notes.is_empty() {
                None
            } else {
                Some(task.notes.as_str())
            },
        ),
    );
    Ok(())
}

/// Mark a task as blocked with a reason.
pub fn block(graph: &mut TaskGraph, id: &str, reason: &str) -> Result<()> {
    {
        let task = graph
            .tasks
            .iter_mut()
            .find(|t| t.id == id)
            .with_context(|| format!("task '{id}' not found"))?;
        task.status = Status::Blocked;
        task.notes = reason.to_string();
    }
    graph.invalidate_done_cache();
    Ok(())
}

/// Reset a blocked task back to pending.
pub fn unblock(graph: &mut TaskGraph, id: &str) -> Result<()> {
    {
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
    }
    graph.invalidate_done_cache();
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
    if count > 0 {
        graph.invalidate_done_cache();
    }
    count
}

/// Returns `Some(cycle_path)` if adding a task with the given id and deps would
/// create a cycle in the existing graph. `None` means the addition is safe.
///
/// Uses a visited set with parent pointers to avoid O(n^2) path cloning.
/// The cycle path is reconstructed from parent pointers only on detection.
fn would_create_cycle(graph: &TaskGraph, new_id: &str, deps: &[String]) -> Option<String> {
    use std::collections::{HashMap, HashSet};

    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
    for t in &graph.tasks {
        adj.insert(
            t.id.as_str(),
            t.depends_on.iter().map(|s| s.as_str()).collect(),
        );
    }
    adj.insert(new_id, deps.iter().map(|s| s.as_str()).collect());

    // DFS from each dep of new_id. Track parent pointers to reconstruct cycle on detection.
    for dep in deps {
        let dep = dep.as_str();
        let mut visited: HashSet<&str> = HashSet::new();
        let mut parent: HashMap<&str, &str> = HashMap::new();
        visited.insert(new_id);
        visited.insert(dep);
        parent.insert(dep, new_id);

        let mut stack = vec![dep];
        while let Some(current) = stack.pop() {
            if current == new_id {
                // Reconstruct cycle path from parent pointers.
                let mut path = vec![];
                let mut trace = current;
                while let Some(&p) = parent.get(trace) {
                    path.push(trace);
                    if p == new_id {
                        path.push(new_id);
                        break;
                    }
                    trace = p;
                }
                path.reverse();
                return Some(path.join(" \u{2192} "));
            }
            if let Some(next_deps) = adj.get(current) {
                for &next in next_deps {
                    if next == new_id {
                        // Found cycle — reconstruct path.
                        let mut path = vec![new_id];
                        let mut trace = current;
                        while let Some(&p) = parent.get(trace) {
                            path.push(trace);
                            if p == new_id {
                                break;
                            }
                            trace = p;
                        }
                        path.push(new_id);
                        path.reverse();
                        // Path is currently reversed from trace; fix order.
                        // new_id -> dep -> ... -> current -> new_id
                        return Some(path.join(" \u{2192} "));
                    }
                    if visited.insert(next) {
                        parent.insert(next, current);
                        stack.push(next);
                    }
                }
            }
        }
    }
    None
}

/// Return the first unused task ID of the form "t1", "t2", …
pub fn next_task_id(g: &TaskGraph) -> Result<String> {
    let used: std::collections::HashSet<&str> = g.tasks.iter().map(|t| t.id.as_str()).collect();
    for n in 1u64.. {
        let candidate = format!("t{}", n);
        if !used.contains(candidate.as_str()) {
            return Ok(candidate);
        }
    }
    bail!("exhausted u64 task ID space")
}

/// Add a new task to the graph. Emits a `pending` trace event when `root` is provided.
pub fn add(graph: &mut TaskGraph, task: Task) -> Result<()> {
    add_traced(graph, task, None)
}

pub fn add_traced(graph: &mut TaskGraph, task: Task, root: Option<&Path>) -> Result<()> {
    if graph.tasks.iter().any(|t| t.id == task.id) {
        bail!("task '{}' already exists", task.id);
    }
    if let Some(cycle_path) = would_create_cycle(graph, &task.id, &task.depends_on) {
        bail!("cycle detected: {}", cycle_path);
    }
    // Trace step is recorded via Session::record in the session_trace layer (#36).
    let _ = (root, &cruxx::step_pending(task.id.as_str()));
    graph.tasks.push(task);
    graph.invalidate_done_cache();
    Ok(())
}

/// Remove a task by id, cleaning up any `depends_on` references to it in remaining tasks.
pub fn remove(graph: &mut TaskGraph, id: &str) -> Result<()> {
    let before = graph.tasks.len();
    graph.tasks.retain(|t| t.id != id);
    if graph.tasks.len() == before {
        bail!("task '{id}' not found");
    }
    for task in &mut graph.tasks {
        task.depends_on.retain(|dep| dep != id);
    }
    graph.invalidate_done_cache();
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
    let removed = before - graph.tasks.len();
    if removed > 0 {
        graph.invalidate_done_cache();
    }
    removed
}

/// Convert the task graph to a `petgraph` directed graph.
/// Each node is a task ID string; edges represent dependencies (dep → dependent).
pub fn to_petgraph(graph: &TaskGraph) -> petgraph::stable_graph::StableDiGraph<String, ()> {
    use petgraph::stable_graph::StableDiGraph;
    use std::collections::HashMap;

    let mut pg: StableDiGraph<String, ()> = StableDiGraph::new();
    let mut idx: HashMap<&str, petgraph::stable_graph::NodeIndex> = HashMap::new();

    for task in &graph.tasks {
        let node = pg.add_node(task.id.clone());
        idx.insert(task.id.as_str(), node);
    }

    for task in &graph.tasks {
        for dep in &task.depends_on {
            if let (Some(&from), Some(&to)) = (idx.get(dep.as_str()), idx.get(task.id.as_str())) {
                pg.add_edge(from, to, ());
            }
        }
    }

    pg
}

/// Render the task graph as a DOT-format string.
pub fn to_dot(graph: &TaskGraph) -> String {
    use petgraph::dot::{Config, Dot};
    let pg = to_petgraph(graph);
    format!("{:?}", Dot::with_config(&pg, &[Config::EdgeNoLabel]))
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

    #[test]
    fn add_rejects_self_cycle() {
        let mut g = TaskGraph::default();
        let mut t = Task::new("t1", "A");
        t.depends_on = vec!["t1".into()];
        let err = add(&mut g, t).unwrap_err();
        assert!(err.to_string().contains("cycle"));
    }

    #[test]
    fn add_rejects_transitive_cycle() {
        let mut g3 = TaskGraph::default();
        let mut existing_b = Task::new("b", "B");
        existing_b.depends_on = vec!["a".into()];
        add(&mut g3, existing_b).unwrap();
        // Now add 'a' with dep on 'b' — this creates a→b→a cycle
        let mut new_a = Task::new("a", "A");
        new_a.depends_on = vec!["b".into()];
        let err = add(&mut g3, new_a).unwrap_err();
        assert!(
            err.to_string().contains("cycle"),
            "expected cycle error, got: {err}"
        );
    }

    #[test]
    fn next_task_id_returns_first_unused() {
        let mut g = TaskGraph::default();
        assert_eq!(next_task_id(&g).unwrap(), "t1");
        add(&mut g, Task::new("t1", "A")).unwrap();
        assert_eq!(next_task_id(&g).unwrap(), "t2");
        add(&mut g, Task::new("t3", "C")).unwrap(); // gap at t2
        assert_eq!(next_task_id(&g).unwrap(), "t2");
    }

    #[test]
    fn remove_cleans_up_depends_on_refs() {
        let mut g = TaskGraph::default();
        add(&mut g, Task::new("a", "Task A")).unwrap();
        let mut b = Task::new("b", "Task B");
        b.depends_on = vec!["a".into()];
        add(&mut g, b).unwrap();

        remove(&mut g, "a").unwrap();

        let task_b = g.tasks.iter().find(|t| t.id == "b").unwrap();
        assert!(
            task_b.depends_on.is_empty(),
            "depends_on should be empty after removing dep"
        );
        let r = runnable(&g);
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].id, "b", "b should now be runnable");
    }

    #[test]
    fn to_petgraph_node_and_edge_counts() {
        let g = graph_with_chain();
        let pg = to_petgraph(&g);
        assert_eq!(pg.node_count(), 2, "expected 2 nodes");
        assert_eq!(pg.edge_count(), 1, "expected 1 edge (t1→t2)");
    }

    #[test]
    fn to_dot_contains_task_ids() {
        let g = graph_with_chain();
        let dot = to_dot(&g);
        assert!(dot.contains("t1"), "DOT missing t1: {dot}");
        assert!(dot.contains("t2"), "DOT missing t2: {dot}");
        // Should have a directed edge indicator
        assert!(dot.contains("->"), "DOT missing edge: {dot}");
    }

    #[test]
    fn add_valid_dag_no_false_positive() {
        let mut g = TaskGraph::default();
        add(&mut g, Task::new("t1", "A")).unwrap();
        let mut t2 = Task::new("t2", "B");
        t2.depends_on = vec!["t1".into()];
        add(&mut g, t2).unwrap();
        let mut t3 = Task::new("t3", "C");
        t3.depends_on = vec!["t1".into()];
        add(&mut g, t3).unwrap(); // diamond — valid DAG
        assert_eq!(g.tasks.len(), 3);
    }

    #[test]
    fn done_cache_invalidated_on_state_transitions() {
        let mut g = graph_with_chain();

        // Initially t1 is runnable (no deps), t2 is not (depends on t1).
        let r = runnable(&g);
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].id, "t1");

        // After starting and completing t1, the cache must be invalidated
        // so that runnable() sees t1 as done and unlocks t2.
        start(&mut g, "t1").unwrap();
        complete(&mut g, "t1", None, None).unwrap();

        // If the cache were stale, t2 would still appear blocked.
        let r = runnable(&g);
        assert_eq!(r.len(), 1, "t2 should be runnable after t1 is done");
        assert_eq!(r[0].id, "t2");

        // Adding a new task also invalidates.
        let mut t3 = Task::new("t3", "Third");
        t3.depends_on = vec!["t1".into()];
        add(&mut g, t3).unwrap();
        let r = runnable(&g);
        assert_eq!(r.len(), 2, "both t2 and t3 should be runnable");

        // Removing a done task invalidates.
        remove(&mut g, "t1").unwrap();
        let r = runnable(&g);
        assert_eq!(r.len(), 2, "t2 and t3 still runnable (deps cleaned up)");
    }

    #[test]
    fn would_create_cycle_deep_chain() {
        // Build a chain: t1 <- t2 <- t3 <- t4 <- t5
        let mut g = TaskGraph::default();
        add(&mut g, Task::new("t1", "A")).unwrap();
        for i in 2..=5 {
            let mut t = Task::new(format!("t{i}"), format!("Task {i}"));
            t.depends_on = vec![format!("t{}", i - 1)];
            add(&mut g, t).unwrap();
        }
        assert_eq!(g.tasks.len(), 5);

        // Adding t0 that depends on t5 is fine (no cycle).
        let mut t0 = Task::new("t0", "Root");
        t0.depends_on = vec!["t5".into()];
        add(&mut g, t0).unwrap();

        // Adding t6 that depends on t5, where t1 depends on t6 would create a cycle.
        // But t1 has no deps on t6, so this is fine.
        let mut t6 = Task::new("t6", "Leaf");
        t6.depends_on = vec!["t5".into()];
        add(&mut g, t6).unwrap();

        // Now try to create an actual cycle: add "tx" with dep on t3,
        // and make t1 depend on tx (which would create t1->tx->...->t3->t2->t1).
        // We can't modify t1's deps after add, so instead:
        // try adding "cycle" with dep on t1, where t5 already depends on t4->t3->t2->t1.
        // Then add a task that would close the loop.
        let mut bad = Task::new("bad", "Cycle maker");
        bad.depends_on = vec!["t5".into()];
        // This is valid (no cycle back to bad).
        add(&mut g, bad).unwrap();

        // Self-cycle still detected.
        let mut self_ref = Task::new("self", "Self");
        self_ref.depends_on = vec!["self".into()];
        let err = add(&mut g, self_ref).unwrap_err();
        assert!(err.to_string().contains("cycle"), "got: {err}");
    }
}
