//! Integration with `hj` — the handoff lifecycle CLI.

use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::detect;

fn project_name(root: &Path) -> Result<String> {
    detect::package_name(root)
}

/// Call `hj handon --project <name>` and return stdout.
pub fn handon(root: &Path) -> Result<String> {
    let project = project_name(root)?;
    let out = Command::new("hj")
        .args(["handon", "--project", &project])
        .current_dir(root)
        .output()
        .context("hj not found on PATH — install hj to enable handoff integration")?;
    if !out.status.success() {
        bail!("hj handon failed: {}", String::from_utf8_lossy(&out.stderr));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Call `hj handoff` with build/test state and return stdout.
pub fn handoff(
    root: &Path,
    build: &str,
    tests: &str,
    summary: &str,
    commits: &[&str],
) -> Result<String> {
    let project = project_name(root)?;
    let mut cmd = Command::new("hj");
    cmd.args(["handoff", "--project", &project])
        .args(["--build", build])
        .args(["--tests", tests])
        .args(["--log-summary", summary])
        .current_dir(root);
    for sha in commits {
        cmd.args(["--commit", sha]);
    }
    let out = cmd
        .output()
        .context("hj not found on PATH — install hj to enable handoff integration")?;
    if !out.status.success() {
        bail!(
            "hj handoff failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}
