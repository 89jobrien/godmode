pub mod cruxx;
pub mod doob;
pub mod gh;
pub mod hj;
pub mod hook_migrate;
pub mod output;
pub mod rx;
pub(crate) mod subprocess;

pub use output::{GraphOut, HandoffOutput, HandonOutput};

use std::path::Path;

use anyhow::Result;

use crate::{graph, model::Status, session};

/// Run the full handon sequence: hj handon + doob next todo + local graph triage.
pub fn handon(root: &Path) -> Result<HandonOutput> {
    let g = graph::load(root)?;
    let summary = g.summary();

    let hj_out = hj::handon(root).ok();
    let next_todo = doob::todo_next_for_root(root).ok().flatten();

    let running_tasks: Vec<String> = g
        .tasks
        .iter()
        .filter(|t| t.status == Status::Running)
        .map(|t| format!("[{}] {}", t.id, t.title))
        .collect();

    let next_runnable = graph::runnable(&g);

    let mut human = String::new();
    if let Some(ref hj) = hj_out {
        human.push_str(hj);
        human.push('\n');
    }
    human.push_str(&format!(
        "=== godmode: {} done, {} running, {} pending, {} blocked ===\n",
        summary.done, summary.running, summary.pending, summary.blocked
    ));
    if !running_tasks.is_empty() {
        human.push_str("In progress:\n");
        for t in &running_tasks {
            human.push_str(&format!("  {}\n", t));
        }
    }
    if !next_runnable.is_empty() {
        human.push_str("Next runnable:\n");
        for t in &next_runnable {
            let crate_tag = t
                .crate_name
                .as_deref()
                .map(|c| format!(" ({})", c))
                .unwrap_or_default();
            human.push_str(&format!("  [{}] {}{}\n", t.id, t.title, crate_tag));
        }
    }
    if let Some(ref todo) = next_todo {
        let title = todo.get("content").and_then(|v| v.as_str()).unwrap_or("?");
        human.push_str(&format!("Next todo (doob): {}\n", title));
    }

    Ok(HandonOutput {
        human,
        graph: GraphOut {
            done: summary.done,
            running: summary.running,
            pending: summary.pending,
            blocked: summary.blocked,
            running_tasks,
        },
        next_todo,
        hj: hj_out,
    })
}

/// Run the full handoff sequence: local graph check + hj handoff.
pub fn handoff(root: &Path) -> Result<HandoffOutput> {
    let summary = session::handoff(root)?;

    let g = graph::load(root)?;
    let running_tasks: Vec<String> = g
        .tasks
        .iter()
        .filter(|t| t.status == Status::Running)
        .map(|t| format!("[{}] {}", t.id, t.title))
        .collect();

    let hj_out = hj::handoff(root, "unknown", "unknown", "session closed", &[]).ok();

    let mut human = String::new();
    if !running_tasks.is_empty() {
        human.push_str(&format!(
            "Warning: {} task(s) still running:\n",
            running_tasks.len()
        ));
        for t in &running_tasks {
            human.push_str(&format!("  {}\n", t));
        }
        human.push_str("Mark them done or blocked before closing.\n");
    }
    if let Some(ref hj) = hj_out {
        human.push_str(hj);
        human.push('\n');
    }
    human.push_str(&format!(
        "Session closed. done={} running={} pending={} blocked={}\n",
        summary.done, summary.running, summary.pending, summary.blocked
    ));

    Ok(HandoffOutput {
        human,
        graph: GraphOut {
            done: summary.done,
            running: summary.running,
            pending: summary.pending,
            blocked: summary.blocked,
            running_tasks,
        },
        hj: hj_out,
    })
}
