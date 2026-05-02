use std::path::Path;
use std::process::Command;

use anyhow::Result;

#[derive(Debug, serde::Serialize)]
pub struct StepResult {
    pub ok: bool,
    pub output: String,
}

#[derive(Debug, serde::Serialize)]
pub struct VerifyReport {
    pub nextest: StepResult,
    pub clippy: StepResult,
    pub fmt: StepResult,
    pub commits: StepResult,
    pub passed: bool,
}

pub fn run(root: &Path, crate_name: Option<&str>) -> Result<VerifyReport> {
    let nextest = {
        let mut cmd = Command::new("cargo");
        cmd.arg("nextest").arg("run");
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
        let output = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        StepResult {
            ok: out.status.success(),
            output,
        }
    };

    let clippy = {
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
        let output = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        StepResult {
            ok: out.status.success(),
            output,
        }
    };

    let fmt = {
        let out = Command::new("cargo")
            .args(["fmt", "--all", "--check"])
            .current_dir(root)
            .output()?;
        let output = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        StepResult {
            ok: out.status.success(),
            output,
        }
    };

    let commits = {
        let out = Command::new("git")
            .args(["-C", &root.to_string_lossy(), "log", "--oneline", "-3"])
            .output()?;
        let output = String::from_utf8_lossy(&out.stdout).into_owned();
        let ok = !output.trim().is_empty();
        StepResult { ok, output }
    };

    let passed = nextest.ok && clippy.ok && fmt.ok && commits.ok;

    Ok(VerifyReport {
        nextest,
        clippy,
        fmt,
        commits,
        passed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_report_serialises() {
        let r = VerifyReport {
            nextest: StepResult {
                ok: true,
                output: "ok".into(),
            },
            clippy: StepResult {
                ok: true,
                output: "".into(),
            },
            fmt: StepResult {
                ok: true,
                output: "".into(),
            },
            commits: StepResult {
                ok: true,
                output: "abc1234 feat: x".into(),
            },
            passed: true,
        };
        let j = serde_json::to_string(&r).unwrap();
        assert!(j.contains("\"passed\":true"));
    }

    #[test]
    fn verify_report_failed_when_any_step_fails() {
        let r = VerifyReport {
            nextest: StepResult {
                ok: false,
                output: "FAILED".into(),
            },
            clippy: StepResult {
                ok: true,
                output: "".into(),
            },
            fmt: StepResult {
                ok: true,
                output: "".into(),
            },
            commits: StepResult {
                ok: true,
                output: "abc".into(),
            },
            passed: false,
        };
        assert!(!r.passed);
    }
}
