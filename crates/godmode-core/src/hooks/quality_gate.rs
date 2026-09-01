//! Fail-fast quality gate: cargo fmt, clippy, nextest.
//!
//! Unlike `verify.rs` which collects all results into a report, this module
//! exits on the first failure — suitable for pre-commit and CI gates.

use std::path::Path;
use std::process::Command;

use anyhow::{Result, bail};

/// Result of running a single gate step.
#[derive(Debug)]
pub struct GateStep {
    /// Display name of the quality check.
    pub name: &'static str,
    /// Whether the quality check completed successfully.
    pub passed: bool,
    /// Combined output produced by the quality check.
    pub output: String,
}

/// Run fmt check, clippy, and nextest in sequence. Bail on first failure.
/// If `crate_name` is Some, scope to that crate; otherwise use --workspace.
pub fn run(root: &Path, crate_name: Option<&str>) -> Result<()> {
    run_fmt(root)?;
    run_clippy(root, crate_name)?;
    run_nextest(root, crate_name)?;
    Ok(())
}

/// Run only fmt + clippy (no tests). Used by pre-commit-gate for speed.
pub fn run_lint_only(root: &Path, crate_name: Option<&str>) -> Result<()> {
    run_fmt(root)?;
    run_clippy(root, crate_name)?;
    Ok(())
}

/// Run cargo fmt --all --check.
pub fn run_fmt(root: &Path) -> Result<()> {
    let out = Command::new("cargo")
        .args(["fmt", "--all", "--check"])
        .current_dir(root)
        .output()?;

    if !out.status.success() {
        let output = combined_output(&out);
        bail!("cargo fmt --check failed:\n{output}");
    }
    Ok(())
}

/// Run cargo clippy with -D warnings.
pub fn run_clippy(root: &Path, crate_name: Option<&str>) -> Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.arg("clippy");
    match crate_name {
        Some(name) => {
            cmd.args(["-p", name]);
        }
        None => {
            cmd.arg("--workspace");
        }
    }
    cmd.args(["--", "-D", "warnings"]);
    cmd.current_dir(root);

    let out = cmd.output()?;
    if !out.status.success() {
        let output = combined_output(&out);
        bail!("cargo clippy failed:\n{output}");
    }
    Ok(())
}

/// Run cargo nextest run.
pub fn run_nextest(root: &Path, crate_name: Option<&str>) -> Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.args(["nextest", "run"]);
    match crate_name {
        Some(name) => {
            cmd.args(["-p", name]);
        }
        None => {
            cmd.arg("--workspace");
        }
    }
    cmd.current_dir(root);

    let out = cmd.output()?;
    if !out.status.success() {
        let output = combined_output(&out);
        bail!("cargo nextest run failed:\n{output}");
    }
    Ok(())
}

fn combined_output(out: &std::process::Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn run_fmt_returns_err_in_nonexistent_dir() {
        let result = run_fmt(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }

    #[test]
    fn combined_output_merges_stdout_stderr() {
        let out = std::process::Output {
            status: std::process::ExitStatus::default(),
            stdout: b"hello ".to_vec(),
            stderr: b"world".to_vec(),
        };
        assert_eq!(combined_output(&out), "hello world");
    }

    // Integration test: runs in the actual workspace
    #[test]
    fn run_fmt_passes_in_workspace() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf();
        // This should pass since we run cargo fmt before committing
        let result = run_fmt(&root);
        assert!(result.is_ok(), "fmt failed: {result:?}");
    }
}
