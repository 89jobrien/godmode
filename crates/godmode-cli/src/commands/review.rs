//! Plugin conformance auditing.

use anyhow::Result;
use godmode_core::review;
use std::path::Path;

use crate::ReviewAction;

pub fn run_review_action(root: &Path, json: bool, sarif: bool, action: ReviewAction) -> Result<()> {
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
