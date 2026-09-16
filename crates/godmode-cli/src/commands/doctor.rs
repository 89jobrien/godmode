use std::path::Path;

use anyhow::Result;

use crate::*;

pub fn handle(command: Cmd, root: &Path, json: bool, _sarif: bool) -> Result<()> {
    match command {
        Cmd::Doctor => {
            use godmode_core::doctor::{RealProbe, run_doctor_at};

            let plugin_root = std::env::var_os("CLAUDE_PLUGIN_ROOT")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| root.to_path_buf());
            let report = run_doctor_at(&RealProbe, Some(&plugin_root));
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                for c in &report.checks {
                    let icon = if c.passed { "ok" } else { "FAIL" };
                    println!("[{icon}] {}: {}", c.name, c.detail);
                }
                if report.all_passed {
                    println!("\nAll checks passed.");
                } else {
                    println!("\nSome checks failed.");
                }
            }
            Ok(())
        }
        _ => unreachable!("dispatcher sent command to the wrong handler"),
    }
}
