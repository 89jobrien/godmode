//! Environment validation.

use anyhow::Result;
use godmode_core::doctor::{RealProbe, run_doctor};

pub fn run_doctor_cmd(json: bool) -> Result<()> {
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
