pub mod crux;
pub mod doob;
pub mod gh;
pub mod handoff_yaml;
pub mod hj;
pub mod hook_migrate;
pub mod hook_runner;
pub mod output;
pub mod rx;
pub(crate) mod subprocess;

pub use output::{GraphOut, HandoffOutput, HandonOutput, PipelineOut};

use std::path::Path;

use anyhow::Result;

use crate::{config::Config, graph, model::Status, pipeline, session};

/// Run the full handon sequence: hj handon + doob next todo + local graph triage.
pub fn handon(root: &Path) -> Result<HandonOutput> {
    build_handon(root)
}

fn build_handon(root: &Path) -> Result<HandonOutput> {
    let cfg = Config::load(root);

    // Ensure sessions dir exists so trace writes don't silently fail
    let _ = std::fs::create_dir_all(crux::sessions_dir(root));

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

    // Load active pipeline state (graceful — None if no pipeline active).
    let pipeline_out = pipeline::load_state(root).ok().flatten().and_then(|state| {
        let p = pipeline::load_pipeline(root, &state.active).ok()?;
        let current_skill = pipeline::current_step(&state, &p)
            .map(|s| s.skill.clone())
            .unwrap_or_else(|| "complete".into());
        let (done, total) = pipeline::progress(&state, &p);
        Some(output::PipelineOut {
            name: state.active.clone(),
            current_skill,
            steps_done: done,
            steps_total: total,
        })
    });

    let human = format_handon_report(
        hj_out.as_deref(),
        &summary,
        &running_tasks,
        &next_runnable,
        next_todo.as_ref(),
        pipeline_out.as_ref(),
    );

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
        pipeline: pipeline_out,
    })
}

/// Run the full handoff sequence: local graph check + hj handoff + dirty tree.
pub fn handoff(root: &Path) -> Result<HandoffOutput> {
    build_handoff(root)
}

fn build_handoff(root: &Path) -> Result<HandoffOutput> {
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

    let mut human =
        format_handoff_report(&running_tasks, &dirty_files, hj_out.as_deref(), &summary);

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
        summary.done, summary.running, summary.pending, summary.blocked,
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
    let root_str = root.to_str().unwrap_or(".");
    subprocess::run(
        "git",
        &["-C", root_str, "status", "--porcelain"],
        "git status",
    )
    .map(|out| {
        out.lines()
            .filter(|l| !l.is_empty())
            .map(str::to_string)
            .collect()
    })
    .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Report formatters (pure string building — no I/O)
// ---------------------------------------------------------------------------

/// Format the handon (session-start) human-readable report.
pub fn format_handon_report(
    hj: Option<&str>,
    summary: &crate::model::GraphSummary,
    running_tasks: &[String],
    next_runnable: &[&crate::model::Task],
    next_todo: Option<&serde_json::Value>,
    pipeline: Option<&output::PipelineOut>,
) -> String {
    let mut out = String::new();
    if let Some(hj) = hj {
        out.push_str(hj);
        out.push('\n');
    }
    out.push_str(&format!(
        "=== godmode: {} done, {} running, {} pending, {} blocked ===\n",
        summary.done, summary.running, summary.pending, summary.blocked
    ));
    if let Some(p) = pipeline {
        out.push_str(&format!(
            "Pipeline: {} ({}/{}) — current: {}\n",
            p.name, p.steps_done, p.steps_total, p.current_skill
        ));
    }
    if !running_tasks.is_empty() {
        out.push_str("In progress:\n");
        for t in running_tasks {
            out.push_str(&format!("  {}\n", t));
        }
    }
    if !next_runnable.is_empty() {
        out.push_str("Next runnable:\n");
        for t in next_runnable {
            let crate_tag = t
                .crate_name
                .as_deref()
                .map(|c| format!(" ({})", c))
                .unwrap_or_default();
            out.push_str(&format!("  [{}] {}{}\n", t.id, t.title, crate_tag));
        }
    }
    if let Some(todo) = next_todo {
        let title = todo.get("content").and_then(|v| v.as_str()).unwrap_or("?");
        out.push_str(&format!("Next todo (doob): {}\n", title));
    }
    out
}

/// Format the handoff (session-end) human-readable report.
pub fn format_handoff_report(
    running_tasks: &[String],
    dirty_files: &[String],
    hj: Option<&str>,
    summary: &crate::model::GraphSummary,
) -> String {
    let mut out = String::new();
    if !running_tasks.is_empty() {
        out.push_str(&format!(
            "Warning: {} task(s) still running:\n",
            running_tasks.len()
        ));
        for t in running_tasks {
            out.push_str(&format!("  {}\n", t));
        }
        out.push_str("Mark them done or blocked before closing.\n");
    }
    if !dirty_files.is_empty() {
        out.push_str(&format!(
            "Warning: {} uncommitted file(s) in working tree:\n",
            dirty_files.len()
        ));
        for f in dirty_files {
            out.push_str(&format!("  {f}\n"));
        }
    }
    if let Some(hj) = hj {
        out.push_str(hj);
        out.push('\n');
    }
    let _ = summary; // counts appended by caller after YAML write
    out
}
