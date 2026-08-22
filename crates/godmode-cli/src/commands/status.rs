//! Graph counts, critical path, and next runnable tasks.

use anyhow::Result;
use godmode_core::{dispatch, graph, model};
use std::path::Path;

pub fn run_status(root: &Path, json: bool, compact: bool) -> Result<()> {
    let g = graph::load(root)?;
    let summary = g.summary();
    let next = graph::runnable(&g);
    let critical = dispatch::critical_path(&g);
    let blocked_tasks: Vec<&model::Task> = g
        .tasks
        .iter()
        .filter(|t| t.status == model::Status::Blocked)
        .collect();
    if json {
        let blocked_detail: Vec<serde_json::Value> = blocked_tasks
            .iter()
            .map(|t| {
                serde_json::json!({
                    "id": t.id,
                    "title": t.title,
                    "reason": t.notes,
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
                "next": next.iter().map(|t| &t.id).collect::<Vec<_>>(),
                "critical_depth": critical.len(),
            }))?
        );
    } else if compact {
        println!(
            "{} done  {} running  {} pending  {} blocked",
            summary.done, summary.running, summary.pending, summary.blocked
        );
        println!("  critical: {} tasks deep", critical.len());
        for t in &next {
            let crate_tag = t
                .crate_name
                .as_deref()
                .map(|c| format!(" ({})", c))
                .unwrap_or_default();
            println!("  next: [{}] {}{}", t.id, t.title, crate_tag);
        }
    } else {
        println!("=== godmode status ===");
        println!("  done     {}", summary.done);
        println!("  running  {}", summary.running);
        println!("  pending  {}", summary.pending);
        if blocked_tasks.is_empty() {
            println!("  blocked  {}", summary.blocked);
        } else {
            let blocked_inline: Vec<String> = blocked_tasks
                .iter()
                .map(|t| {
                    if t.notes.is_empty() {
                        format!("{}: (no reason)", t.id)
                    } else {
                        format!("{}: {}", t.id, t.notes)
                    }
                })
                .collect();
            println!(
                "  blocked  {}  [{}]",
                summary.blocked,
                blocked_inline.join(", ")
            );
        }
        println!();
        if !critical.is_empty() {
            let path_str: Vec<&str> = critical.iter().map(|t| t.id.as_str()).collect();
            println!(
                "  critical path ({} tasks): {}",
                critical.len(),
                path_str.join(" -> ")
            );
        }
        for t in &next {
            let crate_tag = t
                .crate_name
                .as_deref()
                .map(|c| format!(" ({})", c))
                .unwrap_or_default();
            println!("  next: [{}] {}{}", t.id, t.title, crate_tag);
        }
    }
    Ok(())
}
