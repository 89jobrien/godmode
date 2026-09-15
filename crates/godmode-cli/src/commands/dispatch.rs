use std::path::Path;

use anyhow::Result;

use crate::*;

pub fn handle(command: Cmd, root: &Path, json: bool, _sarif: bool) -> Result<()> {
    match command {
        Cmd::Dispatch {
            max,
            critical_path: cp,
        } => {
            let g = graph::load(root)?;
            if cp {
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
        _ => unreachable!("dispatcher sent command to the wrong handler"),
    }
}
