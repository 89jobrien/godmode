//! Integration with `rx` — the script registry CLI.

use std::process::{Command, ExitStatus};

use anyhow::{Context, Result};

/// Run a task's `run:` command.
///
/// If the command starts with `rx:`, delegates to `rx run <script-name>`.
/// Otherwise shells out directly via the system shell.
pub fn run_cmd(run: &str) -> Result<ExitStatus> {
    if let Some(script) = run.strip_prefix("rx:") {
        Command::new("rx")
            .args(["run", script.trim()])
            .status()
            .context("rx not found on PATH")
    } else {
        // Split on whitespace for simple commands; for complex shell expressions
        // use `rx:` prefix with a registered script instead.
        let mut parts = run.split_whitespace();
        let prog = parts.next().unwrap_or(run);
        let args: Vec<&str> = parts.collect();
        Command::new(prog)
            .args(args)
            .status()
            .with_context(|| format!("failed to run: {}", run))
    }
}
