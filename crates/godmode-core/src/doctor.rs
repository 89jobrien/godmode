//! Environment validation checks for `godmode doctor`.

/// Result of a single doctor check.
#[derive(Debug, Clone, serde::Serialize)]
pub struct CheckResult {
    pub name: String,
    pub passed: bool,
    pub detail: String,
}

impl CheckResult {
    fn new(name: String, passed: bool, detail: String) -> Self {
        Self {
            name,
            passed,
            detail,
        }
    }
}

/// Full doctor report.
#[derive(Debug, Clone, serde::Serialize)]
pub struct DoctorReport {
    pub checks: Vec<CheckResult>,
    pub all_passed: bool,
}

/// Port: something that can probe the environment.
pub trait EnvironmentProbe {
    /// Check whether a tool is available on PATH.
    fn has_tool(&self, name: &str) -> bool;
    /// Check 1Password auth status. Returns Ok(()) if authed.
    fn op_authed(&self) -> bool;
    /// List stale git worktrees. Returns list of stale paths (empty = clean).
    fn stale_worktrees(&self) -> Vec<String>;
}

const REQUIRED_TOOLS: &[&str] = &[
    "cargo",
    "git",
    "rustfmt",
    "clippy-driver",
    "op",
    "doob",
    "hj",
];

/// Run all doctor checks against the given probe.
pub fn run_doctor(probe: &dyn EnvironmentProbe) -> DoctorReport {
    let mut checks = Vec::new();

    // Tool checks
    for &tool in REQUIRED_TOOLS {
        let found = probe.has_tool(tool);
        checks.push(CheckResult::new(
            format!("tool:{tool}"),
            found,
            if found {
                "found".into()
            } else {
                "not found on PATH".into()
            },
        ));
    }

    // 1Password auth
    let op_ok = probe.op_authed();
    checks.push(CheckResult::new(
        "op:auth".into(),
        op_ok,
        if op_ok {
            "authenticated".into()
        } else {
            "not authenticated".into()
        },
    ));

    // Stale worktrees
    let stale = probe.stale_worktrees();
    let wt_ok = stale.is_empty();
    checks.push(CheckResult::new(
        "worktrees:stale".into(),
        wt_ok,
        if wt_ok {
            "none".into()
        } else {
            format!("stale: {}", stale.join(", "))
        },
    ));

    let all_passed = checks.iter().all(|c| c.passed);
    DoctorReport { checks, all_passed }
}

/// Real implementation that shells out.
pub struct RealProbe;

impl EnvironmentProbe for RealProbe {
    fn has_tool(&self, name: &str) -> bool {
        which::which(name).is_ok()
    }

    fn op_authed(&self) -> bool {
        std::process::Command::new("op")
            .args(["account", "list"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
    }

    fn stale_worktrees(&self) -> Vec<String> {
        let output = std::process::Command::new("git")
            .args(["worktree", "list", "--porcelain"])
            .output();
        let Ok(out) = output else { return vec![] };
        if !out.status.success() {
            return vec![];
        }
        // Parse porcelain: entries with "prunable" are stale
        let text = String::from_utf8_lossy(&out.stdout);
        let mut stale = Vec::new();
        let mut current_path = String::new();
        for line in text.lines() {
            if let Some(path) = line.strip_prefix("worktree ") {
                current_path = path.to_string();
            }
            if line.starts_with("prunable") && !current_path.is_empty() {
                stale.push(current_path.clone());
            }
        }
        stale
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    struct FakeProbe {
        tools: BTreeMap<String, bool>,
        op_auth: bool,
        stale: Vec<String>,
    }

    impl EnvironmentProbe for FakeProbe {
        fn has_tool(&self, name: &str) -> bool {
            self.tools.get(name).copied().unwrap_or(false)
        }
        fn op_authed(&self) -> bool {
            self.op_auth
        }
        fn stale_worktrees(&self) -> Vec<String> {
            self.stale.clone()
        }
    }

    #[test]
    fn all_pass_when_everything_available() {
        let mut tools = BTreeMap::new();
        for &t in REQUIRED_TOOLS {
            tools.insert(t.to_string(), true);
        }
        let probe = FakeProbe {
            tools,
            op_auth: true,
            stale: vec![],
        };
        let report = run_doctor(&probe);
        assert!(report.all_passed);
        assert!(report.checks.iter().all(|c| c.passed));
    }

    #[test]
    fn fails_when_tool_missing() {
        let mut tools = BTreeMap::new();
        for &t in REQUIRED_TOOLS {
            tools.insert(t.to_string(), t != "doob");
        }
        let probe = FakeProbe {
            tools,
            op_auth: true,
            stale: vec![],
        };
        let report = run_doctor(&probe);
        assert!(!report.all_passed);
        let doob_check = report
            .checks
            .iter()
            .find(|c| c.name == "tool:doob")
            .unwrap();
        assert!(!doob_check.passed);
    }

    #[test]
    fn fails_when_op_not_authed() {
        let mut tools = BTreeMap::new();
        for &t in REQUIRED_TOOLS {
            tools.insert(t.to_string(), true);
        }
        let probe = FakeProbe {
            tools,
            op_auth: false,
            stale: vec![],
        };
        let report = run_doctor(&probe);
        assert!(!report.all_passed);
    }

    #[test]
    fn reports_stale_worktrees() {
        let mut tools = BTreeMap::new();
        for &t in REQUIRED_TOOLS {
            tools.insert(t.to_string(), true);
        }
        let probe = FakeProbe {
            tools,
            op_auth: true,
            stale: vec!["/tmp/stale-wt".into()],
        };
        let report = run_doctor(&probe);
        assert!(!report.all_passed);
        let wt = report
            .checks
            .iter()
            .find(|c| c.name == "worktrees:stale")
            .unwrap();
        assert!(!wt.passed);
        assert!(wt.detail.contains("/tmp/stale-wt"));
    }
}
