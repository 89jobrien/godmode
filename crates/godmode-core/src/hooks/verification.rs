//! verification-before-completion — Stop hook.
//! Warns if godmode verify has not been run since the last commit.

use std::path::Path;

use chrono::{DateTime, Utc};

use crate::integrations::subprocess::command_in;

/// Run the verification hook. Returns a message for stderr (may be empty).
pub fn run(root: &Path) -> String {
    let trace_path = root.join(".ctx/godmode/traces/trace.jsonl");
    if !trace_path.exists() {
        return warn();
    }

    // Get last commit time
    let output = command_in("git", &["log", "-1", "--format=%cI"], root).output();

    let commit_time: DateTime<Utc> = match output {
        Ok(o) if o.status.success() => {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            match s.parse::<DateTime<chrono::FixedOffset>>() {
                Ok(dt) => dt.with_timezone(&Utc),
                Err(_) => return String::new(),
            }
        }
        _ => return String::new(),
    };

    // Look for verify_passed event after the last commit
    let content = match std::fs::read_to_string(&trace_path) {
        Ok(c) => c,
        Err(_) => return warn(),
    };

    let verified = content.lines().any(|line| {
        let val: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => return false,
        };
        let event = val.get("event").and_then(|v| v.as_str()).unwrap_or("");
        if event != "verify_passed" {
            return false;
        }
        let ts_str = val
            .get("timestamp")
            .or_else(|| val.get("ts"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        ts_str
            .parse::<DateTime<chrono::FixedOffset>>()
            .map(|dt| dt.with_timezone(&Utc) > commit_time)
            .unwrap_or(false)
    });

    if verified { String::new() } else { warn() }
}

fn warn() -> String {
    "[godmode:verify] Verification gate not run since last commit — run `godmode verify` before ending session".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::git_harness::{GitRepo, PinnedGitDir};

    /// Regression: the hook must date the last commit of `root`, not of the
    /// repository an ambient `GIT_DIR` points at.
    #[test]
    fn verification_uses_the_given_roots_last_commit_under_a_pinned_git_dir() {
        let fixture = GitRepo::init();
        fixture.commit("fixture commit", Some("2024-01-01T00:00:00+00:00"));
        fixture.write(
            ".ctx/godmode/traces/trace.jsonl",
            "{\"event\":\"verify_passed\",\"timestamp\":\"2022-01-01T00:00:00+00:00\"}\n",
        );

        // The decoy's only commit predates the recorded verify, so a hook that
        // reads it reports "already verified" and stays silent.
        let decoy = GitRepo::init();
        decoy.commit("decoy commit", Some("2020-01-01T00:00:00+00:00"));

        let _pin = PinnedGitDir::pointing_at(&decoy);

        assert!(
            !run(fixture.path()).is_empty(),
            "a commit newer than the last verify must warn"
        );
    }
}
