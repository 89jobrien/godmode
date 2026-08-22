//! Full session context for hooks and subagents.

use anyhow::Result;
use godmode_core::context;
use std::path::Path;

pub fn run_context(root: &Path, json: bool) -> Result<()> {
    let ctx = context::build(root)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&ctx)?);
    } else {
        println!("project: {}", ctx.project);
        if ctx.running.is_empty() {
            println!("running: (none)");
        } else {
            for t in &ctx.running {
                let crate_info = t
                    .crate_name
                    .as_deref()
                    .map(|c| format!(" [{}]", c))
                    .unwrap_or_default();
                println!("running: {} — {}{}", t.id, t.title, crate_info);
            }
        }
        println!("pending: {}", ctx.pending_count);
        if !ctx.blocked.is_empty() {
            for b in &ctx.blocked {
                println!("blocked: {} — {}", b.id, b.reason);
            }
        }
        println!("critical path: {} tasks deep", ctx.critical_path_depth);
        if !ctx.recent_commits.is_empty() {
            println!("recent:");
            for c in &ctx.recent_commits {
                println!("  {c}");
            }
        }
    }
    Ok(())
}
