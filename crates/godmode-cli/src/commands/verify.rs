use std::path::Path;

use anyhow::Result;

use crate::*;

pub fn handle(command: Cmd, root: &Path, json: bool, sarif: bool) -> Result<()> {
    match command {
        Cmd::Verify { crate_name } => {
            let config = godmode_core::config::Config::load(root);
            let report =
                godmode_core::verify::run_with_config(root, crate_name.as_deref(), &config)?;
            if sarif {
                let mut log = godmode_core::sarif::from_verify(&report);
                // Merge rich clippy SARIF (with file locations) as a second run
                let clippy_log = godmode_core::sarif::clippy_sarif(root, crate_name.as_deref())?;
                log.runs.extend(clippy_log.runs);
                // Merge globstar SARIF if available
                if let Some(gs_log) = godmode_core::sarif::globstar_sarif(root) {
                    log.runs.extend(gs_log.runs);
                }
                println!("{}", serde_json::to_string_pretty(&log)?);
            } else if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                let icon = |ok: bool| if ok { "✓" } else { "✗" };
                for step in &report.steps {
                    println!("{:<9}{}", step.name, icon(step.ok));
                }
                if !report.passed {
                    for step in &report.steps {
                        if !step.ok && !step.output.is_empty() {
                            eprintln!("{}", step.output);
                        }
                    }
                }
            }
            if !report.passed {
                std::process::exit(1);
            }
            Ok(())
        }
        _ => unreachable!("dispatcher sent command to the wrong handler"),
    }
}
