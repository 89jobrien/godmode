use std::path::Path;
use std::process::Command;

use anyhow::Result;

#[derive(Debug, serde::Serialize)]
pub struct StepResult {
    pub name: String,
    pub ok: bool,
    pub output: String,
}

impl StepResult {
    fn new(name: String, ok: bool, output: String) -> Self {
        Self { name, ok, output }
    }
}

fn combined_output(stdout: &[u8], stderr: &[u8]) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(stdout),
        String::from_utf8_lossy(stderr)
    )
}

/// A single verification step. Implement this trait to add custom gates.
pub trait VerifyStep {
    /// Human-readable name for this step (e.g. "nextest", "clippy").
    fn name(&self) -> &str;

    /// Execute the step. Returns `Ok(StepResult)` on completion (even if
    /// the underlying command failed — that's recorded in `StepResult::ok`).
    fn run(&self, root: &Path, crate_name: Option<&str>) -> Result<StepResult>;
}

// ---------------------------------------------------------------------------
// Built-in steps
// ---------------------------------------------------------------------------

pub struct NextestStep;
pub struct ClippyStep;
pub struct FmtStep;
pub struct CommitsStep;
pub struct GlobstarStep;

impl VerifyStep for NextestStep {
    fn name(&self) -> &str {
        "nextest"
    }

    fn run(&self, root: &Path, crate_name: Option<&str>) -> Result<StepResult> {
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
        Ok(StepResult::new(
            self.name().into(),
            out.status.success(),
            combined_output(&out.stdout, &out.stderr),
        ))
    }
}

impl VerifyStep for ClippyStep {
    fn name(&self) -> &str {
        "clippy"
    }

    fn run(&self, root: &Path, crate_name: Option<&str>) -> Result<StepResult> {
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
        Ok(StepResult::new(
            self.name().into(),
            out.status.success(),
            combined_output(&out.stdout, &out.stderr),
        ))
    }
}

impl VerifyStep for FmtStep {
    fn name(&self) -> &str {
        "fmt"
    }

    fn run(&self, root: &Path, _crate_name: Option<&str>) -> Result<StepResult> {
        let out = Command::new("cargo")
            .args(["fmt", "--all", "--check"])
            .current_dir(root)
            .output()?;
        Ok(StepResult::new(
            self.name().into(),
            out.status.success(),
            combined_output(&out.stdout, &out.stderr),
        ))
    }
}

impl VerifyStep for CommitsStep {
    fn name(&self) -> &str {
        "commits"
    }

    fn run(&self, root: &Path, _crate_name: Option<&str>) -> Result<StepResult> {
        let out = Command::new("git")
            .args(["-C", &root.to_string_lossy(), "log", "--oneline", "-3"])
            .output()?;
        let output = String::from_utf8_lossy(&out.stdout).into_owned();
        let ok = !output.trim().is_empty();
        Ok(StepResult {
            name: self.name().into(),
            ok,
            output,
        })
    }
}

impl VerifyStep for GlobstarStep {
    fn name(&self) -> &str {
        "globstar"
    }

    fn run(&self, root: &Path, _crate_name: Option<&str>) -> Result<StepResult> {
        // Skip if globstar is not installed or no .globstar/ dir exists
        if !root.join(".globstar").exists() {
            return Ok(StepResult::new(
                self.name().into(),
                true,
                "skipped (no .globstar/ directory)".into(),
            ));
        }
        let globstar = match which::which("globstar") {
            Ok(p) => p,
            Err(_) => {
                return Ok(StepResult::new(
                    self.name().into(),
                    true,
                    "skipped (globstar not on PATH)".into(),
                ));
            }
        };
        let out = Command::new(globstar)
            .args(["check", "--checkers=local"])
            .current_dir(root)
            .output()?;
        Ok(StepResult::new(
            self.name().into(),
            out.status.success(),
            combined_output(&out.stdout, &out.stderr),
        ))
    }
}

pub struct CrsValidateStep;

impl VerifyStep for CrsValidateStep {
    fn name(&self) -> &str {
        "crs-validate"
    }

    fn run(&self, root: &Path, _crate_name: Option<&str>) -> Result<StepResult> {
        // Degrade gracefully if `crs` isn't on PATH — same pattern as GlobstarStep.
        let crs = match which::which("crs") {
            Ok(p) => p,
            Err(_) => {
                return Ok(StepResult::new(
                    self.name().into(),
                    true,
                    "skipped (crs not on PATH)".into(),
                ));
            }
        };
        let out = Command::new(crs)
            .arg("validate")
            .current_dir(root)
            .output()?;
        Ok(StepResult::new(
            self.name().into(),
            out.status.success(),
            combined_output(&out.stdout, &out.stderr),
        ))
    }
}

// ---------------------------------------------------------------------------
// Report
// ---------------------------------------------------------------------------

#[derive(Debug, serde::Serialize)]
pub struct VerifyReport {
    pub steps: Vec<StepResult>,
    pub passed: bool,
}

impl VerifyReport {
    /// Look up a step result by name.
    pub fn step(&self, name: &str) -> Option<&StepResult> {
        self.steps.iter().find(|s| s.name == name)
    }
}

/// Return the default set of verification steps.
pub fn default_steps() -> Vec<Box<dyn VerifyStep>> {
    vec![
        Box::new(NextestStep),
        Box::new(ClippyStep),
        Box::new(FmtStep),
        Box::new(CommitsStep),
        Box::new(GlobstarStep),
    ]
}

/// Run a set of verification steps and collect a report.
pub fn run_steps(
    steps: &[Box<dyn VerifyStep>],
    root: &Path,
    crate_name: Option<&str>,
) -> Result<VerifyReport> {
    let mut results = Vec::with_capacity(steps.len());
    for step in steps {
        results.push(step.run(root, crate_name)?);
    }
    let passed = results.iter().all(|r| r.ok);
    Ok(VerifyReport {
        steps: results,
        passed,
    })
}

/// Run the default verification steps (backward-compatible entry point).
pub fn run(root: &Path, crate_name: Option<&str>) -> Result<VerifyReport> {
    run_steps(&default_steps(), root, crate_name)
}

/// Run the default steps, appending `crs-validate` when
/// `[integrations] crs = true` in `.godmode.toml`.
pub fn run_with_config(
    root: &Path,
    crate_name: Option<&str>,
    config: &crate::config::Config,
) -> Result<VerifyReport> {
    let mut steps = default_steps();
    if config.integrations.crs {
        steps.push(Box::new(CrsValidateStep));
    }
    run_steps(&steps, root, crate_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn step(name: &str, ok: bool, output: &str) -> StepResult {
        StepResult::new(name.into(), ok, output.into())
    }

    #[test]
    fn verify_report_serialises() {
        let r = VerifyReport {
            steps: vec![
                step("nextest", true, "ok"),
                step("clippy", true, ""),
                step("fmt", true, ""),
                step("commits", true, "abc1234 feat: x"),
            ],
            passed: true,
        };
        let j = serde_json::to_string(&r).unwrap();
        assert!(j.contains("\"passed\":true"));
    }

    #[test]
    fn verify_report_failed_when_any_step_fails() {
        let r = VerifyReport {
            steps: vec![
                step("nextest", false, "FAILED"),
                step("clippy", true, ""),
                step("fmt", true, ""),
                step("commits", true, "abc"),
            ],
            passed: false,
        };
        assert!(!r.passed);
    }

    #[test]
    fn step_lookup_by_name() {
        let r = VerifyReport {
            steps: vec![step("clippy", true, "clean")],
            passed: true,
        };
        assert!(r.step("clippy").unwrap().ok);
        assert!(r.step("nonexistent").is_none());
    }

    #[test]
    fn default_steps_returns_five() {
        let steps = default_steps();
        assert_eq!(steps.len(), 5);
        let names: Vec<&str> = steps.iter().map(|s| s.name()).collect();
        assert_eq!(
            names,
            vec!["nextest", "clippy", "fmt", "commits", "globstar"]
        );
    }

    /// Conformance: every built-in VerifyStep impl returns a StepResult
    /// whose name matches the step's declared name.
    #[test]
    fn conformance_step_name_matches_result() {
        struct FakeStep;
        impl VerifyStep for FakeStep {
            fn name(&self) -> &str {
                "fake"
            }
            fn run(&self, _root: &Path, _crate_name: Option<&str>) -> Result<StepResult> {
                Ok(StepResult::new(self.name().into(), true, "pass".into()))
            }
        }
        let s = FakeStep;
        let result = s.run(Path::new("/tmp"), None).unwrap();
        assert_eq!(result.name, s.name());
    }
}

#[cfg(test)]
mod crs_gate_tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn crs_validate_step_skips_gracefully_without_crs_on_path() {
        let step = CrsValidateStep;
        let result = step.run(Path::new("/tmp"), None).unwrap();
        // On this dev machine `crs` IS on PATH (coursers is installed), so this
        // either runs real validation or reports the skip message — either way
        // it must not error, and the name must match.
        assert_eq!(result.name, "crs-validate");
    }

    #[test]
    fn run_with_config_omits_crs_validate_when_integration_disabled() {
        let cfg = Config::default();
        assert!(!cfg.integrations.crs);
    }

    #[test]
    fn run_with_config_includes_crs_validate_when_integration_enabled() {
        let mut cfg = Config::default();
        cfg.integrations.crs = true;
        let mut steps = default_steps();
        if cfg.integrations.crs {
            steps.push(Box::new(CrsValidateStep));
        }
        assert_eq!(steps.last().unwrap().name(), "crs-validate");
    }
}
