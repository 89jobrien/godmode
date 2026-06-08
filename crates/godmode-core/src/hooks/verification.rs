//! verification-before-completion — Stop hook.
//! Warns if godmode verify has not been run since the last commit.

use std::path::Path;

use chrono::{DateTime, Utc};

/// Run the verification hook. Returns a message for stderr (may be empty).
pub fn run(root: &Path) -> String {
    let trace_path = root.join(".ctx/godmode/traces/trace.jsonl");
    if !trace_path.exists() {
        return warn();
    }

    // Get last commit time
    let output = std::process::Command::new("git")
        .args([
            "-C",
            &root.display().to_string(),
            "log",
            "-1",
            "--format=%cI",
        ])
        .output();

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
