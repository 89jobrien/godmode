//! Independent chains and critical path for parallel agent dispatch.

use anyhow::Result;
use godmode_core::{dispatch, graph};
use std::path::Path;

use crate::exit_empty;

pub fn run_dispatch(root: &Path, json: bool, max: usize, critical_path: bool) -> Result<()> {
    let g = graph::load(root)?;
    if critical_path {
        let path = dispatch::critical_path(&g);
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
            for t in &path {
                println!("[{}] {}", t.id, t.title);
            }
        }
    } else {
        let chains = dispatch::independent_chains(&g, max);
        if chains.is_empty() {
            exit_empty(json);
        }
        println!("{}", serde_json::to_string_pretty(&chains)?);
    }
    Ok(())
}
