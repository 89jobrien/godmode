//! GitHub CI triage: classify failures and suggest fixes.

use anyhow::{Context, Result};

#[derive(Debug, serde::Serialize, PartialEq)]
pub enum CiFailureClass {
    CompileError,
    TestFailure,
    ClippyWarning,
    FmtCheck,
    PreCommitHook,
    RunnerEnvironment,
    FalsePositiveDetection,
    DependencyIssue,
    Unknown,
}

#[derive(Debug, serde::Serialize)]
pub struct CiTriageResult {
    pub run_id: String,
    pub class: CiFailureClass,
    pub fix_hint: String,
    /// First 20 lines of failure log.
    pub raw_snippet: String,
}

pub fn classify_log(log: &str) -> CiFailureClass {
    if log.contains("error[E") {
        return CiFailureClass::CompileError;
    }
    if log.contains("FAILED") && (log.contains("assertion") || log.contains("panicked")) {
        return CiFailureClass::TestFailure;
    }
    if log.contains("error: ") && log.contains("-D warnings") {
        return CiFailureClass::ClippyWarning;
    }
    if log.contains("Diff in ") {
        return CiFailureClass::FmtCheck;
    }
    if log.contains("gitleaks") || log.contains("obfsck") || log.contains("coursers") {
        return CiFailureClass::PreCommitHook;
    }
    if log.contains("No such file or directory")
        || log.contains("xcode-select")
        || log.contains("wrong target")
    {
        return CiFailureClass::RunnerEnvironment;
    }
    if log.contains("secret") && log.contains("false positive") {
        return CiFailureClass::FalsePositiveDetection;
    }
    if log.contains("lockfile") || log.contains("yanked") || log.contains("version conflict") {
        return CiFailureClass::DependencyIssue;
    }
    CiFailureClass::Unknown
}

pub fn fix_hint(class: &CiFailureClass) -> &'static str {
    match class {
        CiFailureClass::CompileError => "Fix source error, run: cargo check --workspace",
        CiFailureClass::TestFailure => {
            "Fix implementation or test, run: cargo nextest run --workspace"
        }
        CiFailureClass::ClippyWarning => {
            "Fix warnings, run: cargo clippy --workspace -- -D warnings"
        }
        CiFailureClass::FmtCheck => "Run: cargo fmt --all",
        CiFailureClass::PreCommitHook => "Add allowlist entry for flagged content",
        CiFailureClass::RunnerEnvironment => {
            "Update workflow YAML — check runner, Xcode version, or target triple"
        }
        CiFailureClass::FalsePositiveDetection => "Add .gitleaksignore entry for the flagged path",
        CiFailureClass::DependencyIssue => {
            "Update Cargo.toml/Cargo.lock — check for yanked crates or version conflicts"
        }
        CiFailureClass::Unknown => "Inspect full log: gh run view <run-id> --log-failed",
    }
}

pub fn ci_triage(run_id: Option<&str>) -> Result<CiTriageResult> {
    let id = match run_id {
        Some(id) => id.to_string(),
        None => {
            let out = std::process::Command::new("gh")
                .args([
                    "run",
                    "list",
                    "--limit",
                    "1",
                    "--status",
                    "failure",
                    "--json",
                    "databaseId",
                ])
                .output()
                .context("gh run list failed")?;
            let json: serde_json::Value =
                serde_json::from_slice(&out.stdout).context("parse gh run list JSON")?;
            json.as_array()
                .and_then(|a| a.first())
                .and_then(|v| v.get("databaseId"))
                .and_then(|v| v.as_u64())
                .map(|n| n.to_string())
                .context("no failed run found")?
        }
    };

    let out = std::process::Command::new("gh")
        .args(["run", "view", &id, "--log-failed"])
        .output()
        .context("gh run view failed")?;

    let stdout = String::from_utf8_lossy(&out.stdout);
    let raw_snippet = stdout.lines().take(20).collect::<Vec<_>>().join("\n");
    let class = classify_log(&stdout);
    let hint = fix_hint(&class).to_string();

    Ok(CiTriageResult {
        run_id: id,
        class,
        fix_hint: hint,
        raw_snippet,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_compile_error() {
        assert_eq!(
            classify_log("error[E0308]: mismatched types"),
            CiFailureClass::CompileError
        );
    }

    #[test]
    fn classify_test_failure() {
        assert_eq!(
            classify_log("test FAILED\nassertion failed: left == right"),
            CiFailureClass::TestFailure
        );
    }

    #[test]
    fn classify_test_failure_panicked() {
        assert_eq!(
            classify_log("FAILED\nthread panicked at src/lib.rs:10"),
            CiFailureClass::TestFailure
        );
    }

    #[test]
    fn classify_clippy() {
        assert_eq!(
            classify_log("error: unused variable\n  --> src/lib.rs:5\n  = note: `-D warnings`"),
            CiFailureClass::ClippyWarning
        );
    }

    #[test]
    fn classify_fmt() {
        assert_eq!(
            classify_log("Diff in src/lib.rs:10:"),
            CiFailureClass::FmtCheck
        );
    }

    #[test]
    fn classify_pre_commit_gitleaks() {
        assert_eq!(
            classify_log("gitleaks scan failed"),
            CiFailureClass::PreCommitHook
        );
    }

    #[test]
    fn classify_dependency() {
        assert_eq!(
            classify_log("error: package is yanked"),
            CiFailureClass::DependencyIssue
        );
    }

    #[test]
    fn classify_unknown() {
        assert_eq!(
            classify_log("something completely unrecognised"),
            CiFailureClass::Unknown
        );
    }

    #[test]
    fn fix_hint_non_empty_for_all_classes() {
        let classes = [
            CiFailureClass::CompileError,
            CiFailureClass::TestFailure,
            CiFailureClass::ClippyWarning,
            CiFailureClass::FmtCheck,
            CiFailureClass::PreCommitHook,
            CiFailureClass::RunnerEnvironment,
            CiFailureClass::FalsePositiveDetection,
            CiFailureClass::DependencyIssue,
            CiFailureClass::Unknown,
        ];
        for c in &classes {
            assert!(!fix_hint(c).is_empty(), "fix_hint empty for {:?}", c);
        }
    }
}
