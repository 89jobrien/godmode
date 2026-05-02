//! Integration with `rx` — the script registry CLI.

use std::process::{Command, ExitStatus};

use anyhow::{Context, Result};

// ---------------------------------------------------------------------------
// Pure logic — testable without shelling out
// ---------------------------------------------------------------------------

const SHELL_METACHARACTERS: &[char] = &['|', '>', '<', '&', ';', '$', '`', '(', ')'];

fn needs_shell(run: &str) -> bool {
    run.contains(SHELL_METACHARACTERS)
}

/// Resolve a `run:` string into `(program, args)`.
///
/// `rx:<script>`        → `("rx", ["run", "<script>"])`
/// `echo hi > /tmp/out` → `("sh", ["-c", "echo hi > /tmp/out"])`
/// `cargo test`         → `("cargo", ["test"])`
pub fn resolve_cmd(run: &str) -> (String, Vec<String>) {
    if let Some(script) = run.strip_prefix("rx:") {
        ("rx".into(), vec!["run".into(), script.trim().into()])
    } else if needs_shell(run) {
        ("sh".into(), vec!["-c".into(), run.to_string()])
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
/// Commands containing shell metacharacters are run via `sh -c`.
/// Otherwise shells out directly.
pub fn run_cmd(run: &str) -> Result<ExitStatus> {
    let (prog, args) = resolve_cmd(run);
    Command::new(&prog)
        .args(&args)
        .status()
        .with_context(|| format!("failed to run: {}", run))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_command_splits() {
        let (prog, args) = resolve_cmd("cargo test");
        assert_eq!(prog, "cargo");
        assert_eq!(args, vec!["test"]);
    }

    #[test]
    fn rx_prefix_routes_to_rx() {
        let (prog, args) = resolve_cmd("rx:my-script");
        assert_eq!(prog, "rx");
        assert_eq!(args, vec!["run", "my-script"]);
    }

    #[test]
    fn redirect_uses_shell() {
        let (prog, args) = resolve_cmd("echo hi > /tmp/out");
        assert_eq!(prog, "sh");
        assert_eq!(args, vec!["-c", "echo hi > /tmp/out"]);
    }

    #[test]
    fn pipe_uses_shell() {
        let (prog, args) = resolve_cmd("ls | wc -l");
        assert_eq!(prog, "sh");
        assert_eq!(args, vec!["-c", "ls | wc -l"]);
    }
}
