use std::path::Path;

use anyhow::Result;

use crate::*;

pub fn handle(command: Cmd, _root: &Path, json: bool, _sarif: bool) -> Result<()> {
    match command {
        Cmd::Doctor => {
            use godmode_core::doctor::{RealProbe, run_doctor};

            let report = run_doctor(&RealProbe);
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
