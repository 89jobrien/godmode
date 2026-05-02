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
