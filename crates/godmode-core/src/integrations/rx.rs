//! Integration with `rx` — the script registry CLI.

use std::process::{Command, ExitStatus};

use anyhow::{Context, Result};
use which::which;

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
// rx registry — list and validate
// ---------------------------------------------------------------------------

/// Parse the stdout of `rx list` into script names.
/// Each line is: `name\t-\tbin_path\tsource_path`
pub(crate) fn parse_rx_list_output(output: &str) -> Vec<&str> {
    output
        .lines()
        .filter_map(|line| line.split('\t').next())
        .filter(|name| !name.is_empty())
        .collect()
}

/// Return the names of all scripts registered in the rx registry.
/// Returns an empty vec (not an error) if `rx` is not on PATH.
pub fn list_scripts() -> Result<Vec<String>> {
    if which("rx").is_err() {
        return Ok(vec![]);
    }
    let out = Command::new("rx")
        .arg("list")
        .output()
        .context("failed to run rx list")?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    Ok(parse_rx_list_output(&stdout)
        .into_iter()
        .map(str::to_string)
        .collect())
}

/// Validate that a `run:` field referring to an rx script actually exists.
///
/// - Non-`rx:` strings: always `Ok(())`
/// - `rx:` strings when `rx` not on PATH: `Ok(())` (graceful degradation)
/// - `rx:` strings when script not found: `Err(...)`
pub fn validate_run(run: &str) -> Result<()> {
    let Some(script) = run.strip_prefix("rx:") else {
        return Ok(());
    };
    let script = script.trim();
    let scripts = list_scripts()?;
    if scripts.is_empty() {
        return Ok(());
    }
    if scripts.iter().any(|s| s == script) {
        Ok(())
    } else {
        anyhow::bail!(
            "rx script '{}' not found in registry (rx list returned {} scripts)",
            script,
            scripts.len()
        )
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
    fn validate_run_passes_for_non_rx_command() {
        assert!(validate_run("cargo test").is_ok());
    }

    #[test]
    fn list_scripts_parses_tab_separated_output() {
        let raw = "foo\t-\t/bin/foo\t/src/foo.nu\nbar\t-\t/bin/bar\t/src/bar.nu\n";
        let names = parse_rx_list_output(raw);
        assert_eq!(names, vec!["foo", "bar"]);
    }

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
