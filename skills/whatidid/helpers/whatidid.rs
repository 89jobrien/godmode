#!/usr/bin/env rust-script
//! whatidid — daily Claude Code activity report.
//!
//! Usage: whatidid.rs [YYYY-MM-DD] [--no-open]
//!
//! Orchestrates harvest → analyze → report pipeline.
//! Requires ANTHROPIC_API_KEY (inject via: op run --env-file=... -- whatidid.rs).
//!
//! ```cargo
//! [dependencies]
//! anyhow = "1"
//! chrono = "0.4"
//! ```

use anyhow::{Context, Result};
use chrono::Local;
use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let date_str = args
        .next()
        .filter(|a| !a.starts_with('-'))
        .unwrap_or_else(|| Local::now().format("%Y-%m-%d").to_string());
    let no_open = std::env::args().any(|a| a == "--no-open");

    let helpers = helpers_dir()?;

    // Step 1: harvest
    eprintln!("[1/3] harvesting sessions for {date_str}...");
    let harvest = run_rust_script(
        &helpers.join("harvest.rs"),
        &[&date_str],
    )?;
    if harvest.trim().is_empty() || harvest.trim() == "[]" {
        eprintln!("No Claude Code sessions found for {date_str}.");
        eprintln!(
            "Sessions are stored under ~/.claude/projects/<project>/*.jsonl"
        );
        return Ok(());
    }

    // Write harvest to temp file
    let sessions_path = format!("/tmp/whatidid-sessions-{date_str}.json");
    std::fs::write(&sessions_path, &harvest)
        .context("write sessions temp file")?;

    // Step 2: analyze
    eprintln!("[2/3] analyzing with Claude...");
    let digest = run_rust_script(
        &helpers.join("analyze.rs"),
        &[&sessions_path, &date_str],
    )?;

    let digest_path = format!("/tmp/whatidid-digest-{date_str}.json");
    std::fs::write(&digest_path, &digest)
        .context("write digest temp file")?;

    // Step 3: report
    let open_flag = if no_open { "--no-open" } else { "" };
    let report_args: Vec<&str> = if no_open {
        vec![&digest_path, &date_str, open_flag]
    } else {
        vec![&digest_path, &date_str]
    };

    eprintln!("[3/3] rendering report...");
    let summary = run_rust_script(&helpers.join("report.rs"), &report_args)?;
    if !summary.trim().is_empty() {
        println!("{summary}");
    }

    Ok(())
}

fn run_rust_script(script: &PathBuf, args: &[&str]) -> Result<String> {
    let mut cmd = Command::new("rust-script");
    cmd.arg(script);
    for arg in args.iter().filter(|a| !a.is_empty()) {
        cmd.arg(arg);
    }
    cmd.stdout(Stdio::piped()).stderr(Stdio::inherit());

    let output = cmd
        .output()
        .with_context(|| format!("run rust-script {}", script.display()))?;

    if !output.status.success() {
        anyhow::bail!(
            "rust-script {} failed (exit {})",
            script.display(),
            output.status
        );
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn helpers_dir() -> Result<PathBuf> {
    // Resolve relative to this script's canonical location
    let home = std::env::var("HOME").context("HOME not set")?;
    Ok(PathBuf::from(home)
        .join("dev/minibox/.claude/skills/whatidid/helpers"))
}
