//! Integration with `hj` — the handoff lifecycle CLI.

use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::detect;

// ---------------------------------------------------------------------------
// Pure logic — testable without shelling out
// ---------------------------------------------------------------------------

/// Build the argv for `hj handoff` (excluding the binary name itself).
pub fn build_handoff_args(
    project: &str,
    build: &str,
    tests: &str,
    summary: &str,
    commits: &[&str],
) -> Vec<String> {
    let mut args = vec![
        "handoff".into(),
        "--project".into(),
        project.into(),
        "--build".into(),
        build.into(),
        "--tests".into(),
        tests.into(),
        "--log-summary".into(),
        summary.into(),
    ];
    for sha in commits {
        args.push("--commit".into());
        args.push((*sha).into());
    }
    args
}

// ---------------------------------------------------------------------------
// Shell-out layer
// ---------------------------------------------------------------------------

/// Call `hj handon --project <name>` and return stdout.
pub fn handon(root: &Path) -> Result<String> {
    let project = detect::package_name(root)?;
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
    let project = detect::package_name(root)?;
    let args = build_handoff_args(&project, build, tests, summary, commits);
    let out = Command::new("hj")
        .args(&args)
        .current_dir(root)
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
