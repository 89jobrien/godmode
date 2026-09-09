//! Workspace status, dispatch, and graph rendering adapters.

use std::io::Write;
use std::path::Path;

use anyhow::Result;
use godmode_core::{context, dispatch, graph, model};

use super::exit_empty;

pub(super) fn run_context(root: &Path, json: bool) -> Result<()> {
    let ctx = context::build(root)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&ctx)?);
        return Ok(());
    }

    println!("project: {}", ctx.project);
    if ctx.running.is_empty() {
        println!("running: (none)");
    } else {
        for task in &ctx.running {
            let crate_info = task
                .crate_name
                .as_deref()
                .map(|name| format!(" [{name}]"))
                .unwrap_or_default();
            println!("running: {} — {}{}", task.id, task.title, crate_info);
        }
    }
    println!("pending: {}", ctx.pending_count);
    for blocked in &ctx.blocked {
        println!("blocked: {} — {}", blocked.id, blocked.reason);
    }
    println!("critical path: {} tasks deep", ctx.critical_path_depth);
    if !ctx.recent_commits.is_empty() {
        println!("recent:");
        for commit in &ctx.recent_commits {
            println!("  {commit}");
        }
    }
    Ok(())
}

pub(super) fn run_status(root: &Path, json: bool, compact: bool) -> Result<()> {
    let graph = graph::load(root)?;
    let summary = graph.summary();
    let next = graph::runnable(&graph);
    let critical = dispatch::critical_path(&graph);
    let blocked: Vec<&model::Task> = graph
        .tasks
        .iter()
        .filter(|task| task.status == model::Status::Blocked)
        .collect();

    if json {
        let blocked_detail: Vec<serde_json::Value> = blocked
            .iter()
            .map(|task| {
                serde_json::json!({
                    "id": task.id,
                    "title": task.title,
                    "reason": task.notes,
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "done": summary.done,
                "running": summary.running,
                "pending": summary.pending,
                "blocked": summary.blocked,
                "blocked_detail": blocked_detail,
                "next": next.iter().map(|task| &task.id).collect::<Vec<_>>(),
                "critical_depth": critical.len(),
            }))?
        );
    } else if compact {
        println!(
            "{} done  {} running  {} pending  {} blocked",
            summary.done, summary.running, summary.pending, summary.blocked
        );
        println!("  critical: {} tasks deep", critical.len());
        render_next_tasks(&next);
    } else {
        println!("=== godmode status ===");
        println!("  done     {}", summary.done);
        println!("  running  {}", summary.running);
        println!("  pending  {}", summary.pending);
        if blocked.is_empty() {
            println!("  blocked  {}", summary.blocked);
        } else {
            let details = blocked
                .iter()
                .map(|task| {
                    if task.notes.is_empty() {
                        format!("{}: (no reason)", task.id)
                    } else {
                        format!("{}: {}", task.id, task.notes)
                    }
                })
                .collect::<Vec<_>>();
            println!("  blocked  {}  [{}]", summary.blocked, details.join(", "));
        }
        println!();
        if !critical.is_empty() {
            let path = critical
                .iter()
                .map(|task| task.id.as_str())
                .collect::<Vec<_>>();
            println!(
                "  critical path ({} tasks): {}",
                critical.len(),
                path.join(" -> ")
            );
        }
        render_next_tasks(&next);
    }
    Ok(())
}

fn render_next_tasks(tasks: &[&model::Task]) {
    for task in tasks {
        let crate_tag = task
            .crate_name
            .as_deref()
            .map(|name| format!(" ({name})"))
            .unwrap_or_default();
        println!("  next: [{}] {}{}", task.id, task.title, crate_tag);
    }
}

pub(super) fn run_dispatch(root: &Path, json: bool, max: usize, critical_path: bool) -> Result<()> {
    let graph = graph::load(root)?;
    if critical_path {
        let path = dispatch::critical_path(&graph);
        if path.is_empty() {
            exit_empty(json);
        }
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "critical_path": path,
                    "depth": path.len(),
                }))?
            );
        } else {
            println!("=== critical path ({} tasks) ===", path.len());
            for task in &path {
                println!("[{}] {}", task.id, task.title);
            }
        }
    } else {
        let chains = dispatch::independent_chains(&graph, max);
        if chains.is_empty() {
            exit_empty(json);
        }
        println!("{}", serde_json::to_string_pretty(&chains)?);
    }
    Ok(())
}

pub(super) fn run_visualize_graph(
    root: &Path,
    json: bool,
    format: &str,
    output_path: Option<String>,
) -> Result<()> {
    let graph = graph::load(root)?;
    let dot = graph::to_dot(&graph);
    let content = match format {
        "dot" => dot,
        "svg" => render_svg(&dot)?,
        other => anyhow::bail!("unsupported format '{other}'; expected dot or svg"),
    };
    if let Some(path) = output_path {
        std::fs::write(&path, &content)?;
        if json {
            println!("{}", serde_json::json!({"path": path}));
        } else {
            println!("wrote {path}");
        }
    } else {
        print!("{content}");
    }
    Ok(())
}

fn render_svg(dot: &str) -> Result<String> {
    let child = std::process::Command::new("dot")
        .args(["-Tsvg"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .spawn();
    let Ok(mut child) = child else {
        eprintln!("warning: graphviz `dot` not found — falling back to DOT format");
        return Ok(dot.to_owned());
    };
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(dot.as_bytes());
    }
    let output = child.wait_with_output()?;
    anyhow::ensure!(
        output.status.success(),
        "graphviz dot exited with {}",
        output.status
    );
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}
