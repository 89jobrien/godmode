//! Integration with `rx` — the script registry CLI.

use std::process::{Command, ExitStatus};

use anyhow::{Context, Result};

// ---------------------------------------------------------------------------
// Pure logic — testable without shelling out
// ---------------------------------------------------------------------------

/// Resolve a `run:` string into `(program, args)`.
///
/// `rx:<script>` → `("rx", ["run", "<script>"])`
/// `cargo test`  → `("cargo", ["test"])`
pub fn resolve_cmd(run: &str) -> (String, Vec<String>) {
    if let Some(script) = run.strip_prefix("rx:") {
        ("rx".into(), vec!["run".into(), script.trim().into()])
    } else {
        let mut parts = run.split_whitespace();
        let prog = parts.next().unwrap_or(run).to_string();
        let args = parts.map(str::to_string).collect();
        (prog, args)
    }
}

// ---------------------------------------------------------------------------
// Shell-out layer
// ---------------------------------------------------------------------------

/// Run a task's `run:` command.
///
/// If the command starts with `rx:`, delegates to `rx run <script-name>`.
/// Otherwise shells out directly.
pub fn run_cmd(run: &str) -> Result<ExitStatus> {
    let (prog, args) = resolve_cmd(run);
    Command::new(&prog)
        .args(&args)
        .status()
        .with_context(|| format!("failed to run: {}", run))
}
