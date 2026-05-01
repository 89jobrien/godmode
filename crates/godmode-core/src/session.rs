use std::path::Path;

use anyhow::Result;

use crate::graph;
use crate::model::{GraphSummary, Status};

/// Print a triage summary to stdout. Called at session start.
pub fn handon(root: &Path) -> Result<()> {
    let graph = graph::load(root)?;
    if graph.tasks.is_empty() {
        println!("No tasks. Run `godmode plan ingest <plan>` or `godmode task add`.");
        return Ok(());
    }

    let summary = graph.summary();
    println!(
        "Tasks: {} done, {} running, {} pending, {} blocked",
        summary.done, summary.running, summary.pending, summary.blocked
    );

    // Show running tasks first.
    let running: Vec<_> = graph
        .tasks
        .iter()
        .filter(|t| t.status == Status::Running)
        .collect();
    if !running.is_empty() {
        println!("\nIn progress:");
        for t in &running {
            println!("  [{}] {}", t.id, t.title);
        }
    }

    // Show next runnable.
    let next = graph::runnable(&graph);
    if !next.is_empty() {
        println!("\nNext runnable:");
        for t in &next {
            let crate_tag = t
                .crate_name
                .as_deref()
                .map(|c| format!(" ({})", c))
                .unwrap_or_default();
            println!("  [{}] {}{}", t.id, t.title, crate_tag);
        }
    }

    // Show blocked tasks.
    let blocked: Vec<_> = graph
        .tasks
        .iter()
        .filter(|t| t.status == Status::Blocked)
        .collect();
    if !blocked.is_empty() {
        println!("\nBlocked:");
        for t in &blocked {
            println!("  [{}] {} — {}", t.id, t.title, t.notes);
        }
    }

    Ok(())
}

/// Validate that no tasks are left in `running` state at session end.
pub fn handoff(root: &Path) -> Result<GraphSummary> {
    let graph = graph::load(root)?;
    let running: Vec<_> = graph
        .tasks
        .iter()
        .filter(|t| t.status == Status::Running)
        .collect();
    if !running.is_empty() {
        eprintln!("Warning: {} task(s) still running:", running.len());
        for t in &running {
            eprintln!("  [{}] {}", t.id, t.title);
        }
        eprintln!("Mark them done or blocked before closing.");
    }
    Ok(graph.summary())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph;
    use crate::model::Task;
    use tempfile::TempDir;

    #[test]
    fn handon_empty_graph_prints_message() {
        let dir = TempDir::new().unwrap();
        // No task file — should succeed with empty message.
        handon(dir.path()).unwrap();
    }

    #[test]
    fn handoff_warns_on_running_tasks() {
        let dir = TempDir::new().unwrap();
        let mut g = crate::model::TaskGraph::default();
        let mut t = Task::new("t1", "Unfinished");
        t.status = Status::Running;
        g.tasks.push(t);
        graph::save(dir.path(), &g).unwrap();
        let summary = handoff(dir.path()).unwrap();
        assert_eq!(summary.running, 1);
    }
}
