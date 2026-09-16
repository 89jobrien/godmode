use std::path::Path;

use anyhow::Result;

use crate::*;

pub fn handle(command: Cmd, _root: &Path, json: bool, _sarif: bool) -> Result<()> {
    match command {
        Cmd::Init => {
            use godmode_core::doctor::RealProbe;
            use godmode_core::init::{RealFs, run_init};

            let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
            let global_dir = std::path::PathBuf::from(&home)
                .join(".config")
                .join("godmode");
            let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            let report = run_init(&RealFs, &RealProbe, &cwd, &global_dir)?;

            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                if report.global_created {
                    println!("Created global config: {}", report.global_path.display());
                } else {
                    println!(
                        "Global config already exists: {}",
                        report.global_path.display()
                    );
                }
                if report.project_created {
                    if let Some(ref p) = report.project_path {
                        println!("Created project state: {}", p.display());
                    }
                } else if report.project_path.is_some() {
                    println!("Project state already exists.");
                } else {
                    println!("No Rust project detected (no Cargo.toml found).");
                }
                if report.gitignore_updated {
                    println!("Added .ctx/ to .gitignore");
                }
                println!();
                println!("Doctor:");
                for c in &report.doctor.checks {
                    let icon = if c.passed { "ok" } else { "FAIL" };
                    println!("  [{icon}] {}: {}", c.name, c.detail);
                }
            }
            Ok(())
        }
        _ => unreachable!("dispatcher sent command to the wrong handler"),
    }
}
