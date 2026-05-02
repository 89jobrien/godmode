//! Integration with `hj` — the handoff lifecycle CLI.

use std::path::Path;

use anyhow::Result;

use crate::detect;
use crate::integrations::subprocess;

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
    subprocess::run_in(
        "hj",
        &["handon", "--project", &project],
        root,
        "install hj to enable handoff integration",
    )
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
    let args_ref: Vec<&str> = args.iter().map(String::as_str).collect();
    subprocess::run_in(
        "hj",
        &args_ref,
        root,
        "install hj to enable handoff integration",
    )
}
