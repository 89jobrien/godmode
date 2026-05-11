pub mod cruxx;
pub mod doob;
pub mod gh;
pub mod handoff_yaml;
pub mod hj;
pub mod hook_migrate;
pub mod hook_runner;
pub mod output;
pub mod rx;
pub(crate) mod subprocess;

pub use output::{GraphOut, HandoffOutput, HandonOutput};

use std::path::Path;

use anyhow::Result;

use crate::{config::Config, graph, model::Status, session};

/// Run the full handon sequence: hj handon + doob next todo + local graph triage.
pub fn handon(root: &Path) -> Result<HandonOutput> {
    let cfg = Config::load(root);

    // Ensure sessions dir exists so trace writes don't silently fail
    let _ = std::fs::create_dir_all(cruxx::sessions_dir(root));

    let g = graph::load(root)?;
    let summary = g.summary();

    let hj_out = if cfg.integrations.hj {
        hj::handon(root).ok()
    } else {
        None
    };
    let next_todo = if cfg.integrations.doob {
        doob::todo_next_for_root(root).ok().flatten()
    } else {
        None
    };

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

/// Run the full handoff sequence: local graph check + hj handoff + dirty tree.
pub fn handoff(root: &Path) -> Result<HandoffOutput> {
    let cfg = Config::load(root);
    let summary = session::handoff(root)?;

    let g = graph::load(root)?;
    let running_tasks: Vec<String> = g
        .tasks
        .iter()
        .filter(|t| t.status == Status::Running)
        .map(|t| format!("[{}] {}", t.id, t.title))
        .collect();

    let hj_out = if cfg.integrations.hj {
        hj::handoff(root, "unknown", "unknown", "session closed", &[]).ok()
    } else {
        None
    };

    // Detect uncommitted/untracked files
    let dirty_files = detect_dirty_files(root);

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
    if !dirty_files.is_empty() {
        human.push_str(&format!(
            "Warning: {} uncommitted file(s) in working tree:\n",
            dirty_files.len()
        ));
        for f in &dirty_files {
            human.push_str(&format!("  {f}\n"));
        }
    }
    if let Some(ref hj) = hj_out {
        human.push_str(hj);
        human.push('\n');
    }
    // Write native HANDOFF YAML from task graph state, then sync to doob db
    if cfg.handoff.enabled {
        let session_summary = format!(
            "done={} running={} pending={} blocked={}",
            summary.done, summary.running, summary.pending, summary.blocked
        );
        if let Ok((handoff_path, item_ids)) =
            handoff_yaml::write_handoff(root, &g.tasks, &dirty_files, &session_summary, &cfg)
            && cfg.handoff.doob_sync
            && cfg.integrations.doob
        {
            let project = cfg.project_name(root);
            let _ = doob::handoff_sync(&handoff_path, &project, &item_ids);
        }
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
        dirty_files,
    })
}

/// Run `git status --porcelain` and return the list of dirty files.
fn detect_dirty_files(root: &Path) -> Vec<String> {
    let out = std::process::Command::new("git")
        .args(["-C", root.to_str().unwrap_or("."), "status", "--porcelain"])
        .output();
    match out {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout)
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| l.to_string())
            .collect(),
        _ => vec![],
    }
}
