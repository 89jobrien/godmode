use std::path::Path;

use anyhow::Result;

use crate::*;

pub fn handle(command: Cmd, root: &Path, json: bool, sarif: bool) -> Result<()> {
    match command {
        Cmd::Review { action } => {
            let report = match action {
                ReviewAction::Self_ => review::run_all(root)?,
                ReviewAction::Skills => review::check_skills(root)?,
                ReviewAction::Agents => review::check_agents(root)?,
            };
            if sarif {
                let log = godmode_core::sarif::from_review(&report);
                println!("{}", serde_json::to_string_pretty(&log)?);
            } else if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else if report.passed {
                println!("{} checks passed.", report.checks);
            } else {
                for f in &report.findings {
                    println!("{}", f.message);
                }
                println!(
                    "\n{} checks failed out of {} total.",
                    report.findings.len(),
                    report.checks
                );
            }
            if !report.passed {
                std::process::exit(1);
            }
            Ok(())
        }
        _ => unreachable!("dispatcher sent command to the wrong handler"),
    }
}
