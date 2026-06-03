//! Shared subprocess execution helper for integration shell-outs.

use std::process::Command;

use anyhow::{Context, Result, bail};

/// Run `bin` with `args`, returning stdout on success.
///
/// On failure returns an error with `context_msg` prepended.
/// If the binary is not on PATH, the error message includes `bin not found on PATH`.
pub fn run(bin: &str, args: &[&str], context_msg: &str) -> Result<String> {
    let out = Command::new(bin)
        .args(args)
        .output()
        .with_context(|| format!("{bin} not found on PATH — {context_msg}"))?;
    if !out.status.success() {
        bail!(
            "{bin} {cmd} failed: {stderr}",
            cmd = args.first().copied().unwrap_or(""),
            stderr = String::from_utf8_lossy(&out.stderr),
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Run `bin` with `args` in `cwd`, returning stdout on success.
pub fn run_in(
    bin: &str,
    args: &[&str],
    cwd: &std::path::Path,
    context_msg: &str,
) -> Result<String> {
    let out = Command::new(bin)
        .args(args)
        .current_dir(cwd)
        .output()
        .with_context(|| format!("{bin} not found on PATH — {context_msg}"))?;
    if !out.status.success() {
        bail!(
            "{bin} {cmd} failed: {stderr}",
            cmd = args.first().copied().unwrap_or(""),
            stderr = String::from_utf8_lossy(&out.stderr),
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_echo_returns_stdout() {
        let out = run("echo", &["hello"], "echo test").unwrap();
        assert_eq!(out.trim(), "hello");
    }

    #[test]
    fn run_missing_binary_returns_error() {
        let err = run("__nonexistent_bin_xyz__", &[], "test").unwrap_err();
        assert!(
            err.to_string().contains("not found on PATH"),
            "error should mention PATH: {}",
            err
        );
    }

    #[test]
    fn run_failing_command_returns_error() {
        let err = run("false", &[], "test false").unwrap_err();
        assert!(err.to_string().contains("failed"));
    }

    #[test]
    fn run_in_respects_cwd() {
        let dir = tempfile::TempDir::new().unwrap();
        let out = run_in("pwd", &[], dir.path(), "pwd test").unwrap();
        let expected = std::fs::canonicalize(dir.path()).unwrap();
        let actual = std::fs::canonicalize(out.trim()).unwrap();
        assert_eq!(actual, expected);
    }
}
