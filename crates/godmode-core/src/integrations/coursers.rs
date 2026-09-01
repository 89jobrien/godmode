//! Integration with `coursers` — reads failure-learning state to surface
//! frequently-failing commands in `godmode handon` and `godmode context`.
//!
//! Reads `course-correct-state.json` directly (no subprocess). Missing or
//! malformed state files are treated as "no data" — never an error.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Minimum failure count within the window before a command is surfaced.
const FAILURE_THRESHOLD: usize = 3;

/// Lookback window in seconds — matches coursers' default failure-learning window.
const WINDOW_SECS: u64 = 3600;

/// A single failing command, summarized for display.
#[derive(Debug, Serialize)]
pub struct FailingSummary {
    /// Truncated command text recorded by coursers.
    pub command_preview: String,
    /// Number of failures observed within the lookback window.
    pub count: usize,
    /// Number of seconds elapsed since the command most recently failed.
    pub last_seen_ago_secs: u64,
}

#[derive(Debug, Default, Deserialize)]
struct FailureEntry {
    #[serde(default)]
    command_preview: String,
    #[serde(default)]
    timestamps: Vec<u64>,
    #[serde(default)]
    last_seen: f64,
}

#[derive(Debug, Default, Deserialize)]
struct State {
    #[serde(default)]
    failures: std::collections::HashMap<String, FailureEntry>,
}

/// Resolve the coursers failure-learning state file: project-local first,
/// falling back to the global config path.
fn state_path(root: &Path) -> PathBuf {
    let local = root.join(".ctx/course-correct-state.json");
    if local.exists() {
        return local;
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".config/coursers/course-correct-state.json")
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Read coursers failure-learning state and return commands with 3+ failures
/// in the last hour. Returns an empty vec on any missing/malformed file.
pub fn failing_commands(root: &Path) -> Vec<FailingSummary> {
    failing_commands_from_path(&state_path(root))
}

fn failing_commands_from_path(path: &Path) -> Vec<FailingSummary> {
    let Ok(raw) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    let Ok(state) = serde_json::from_str::<State>(&raw) else {
        return Vec::new();
    };

    let now = now_secs();
    let mut out: Vec<FailingSummary> = state
        .failures
        .values()
        .filter_map(|entry| {
            let recent_count = entry
                .timestamps
                .iter()
                .filter(|&&ts| now.saturating_sub(ts) <= WINDOW_SECS)
                .count();
            if recent_count < FAILURE_THRESHOLD {
                return None;
            }
            let last_seen_ago_secs = now.saturating_sub(entry.last_seen as u64);
            Some(FailingSummary {
                command_preview: entry.command_preview.clone(),
                count: recent_count,
                last_seen_ago_secs,
            })
        })
        .collect();

    out.sort_by_key(|f| std::cmp::Reverse(f.count));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_state(dir: &Path, json: &str) -> PathBuf {
        let path = dir.join("state.json");
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(json.as_bytes()).unwrap();
        path
    }

    #[test]
    fn missing_file_returns_empty() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("does-not-exist.json");
        assert!(failing_commands_from_path(&path).is_empty());
    }

    #[test]
    fn malformed_file_returns_empty() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = write_state(dir.path(), "not json");
        assert!(failing_commands_from_path(&path).is_empty());
    }

    #[test]
    fn below_threshold_excluded() {
        let dir = tempfile::TempDir::new().unwrap();
        let now = now_secs();
        let json = format!(
            r#"{{"failures":{{"abc":{{"command_preview":"grep foo","timestamps":[{now},{now}],"last_seen":{now}.0}}}}}}"#,
        );
        let path = write_state(dir.path(), &json);
        assert!(failing_commands_from_path(&path).is_empty());
    }

    #[test]
    fn at_threshold_included() {
        let dir = tempfile::TempDir::new().unwrap();
        let now = now_secs();
        let json = format!(
            r#"{{"failures":{{"abc":{{"command_preview":"grep foo","timestamps":[{now},{now},{now}],"last_seen":{now}.0}}}}}}"#,
        );
        let path = write_state(dir.path(), &json);
        let out = failing_commands_from_path(&path);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].command_preview, "grep foo");
        assert_eq!(out[0].count, 3);
    }

    #[test]
    fn entries_outside_window_excluded() {
        let dir = tempfile::TempDir::new().unwrap();
        let now = now_secs();
        let old = now.saturating_sub(WINDOW_SECS + 100);
        let json = format!(
            r#"{{"failures":{{"abc":{{"command_preview":"grep foo","timestamps":[{old},{old},{old}],"last_seen":{old}.0}}}}}}"#,
        );
        let path = write_state(dir.path(), &json);
        assert!(failing_commands_from_path(&path).is_empty());
    }
}
