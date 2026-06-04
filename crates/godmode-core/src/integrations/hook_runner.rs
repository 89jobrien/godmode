//! Hook observability: append hook run events to `.ctx/godmode/traces/hooks.log`.
//!
//! All writes are non-fatal — errors are silently dropped so a missing `.ctx/`
//! directory or a read-only filesystem never aborts the caller.

use std::path::Path;

/// Append a single hook run event to `.ctx/godmode/traces/hooks.log` as a JSONL line.
///
/// Fields:
/// - `hook`      — name/identifier of the hook script
/// - `event`     — the Claude hook event type (e.g. "PreToolUse/Bash")
/// - `exit_code` — process exit code
/// - `stderr`    — first 120 chars of stderr output
/// - `ts`        — ISO 8601 timestamp (local time)
pub fn append_hook_event(
    root: &Path,
    hook: &str,
    event: &str,
    exit_code: i32,
    stderr: &str,
) -> std::io::Result<()> {
    use std::io::Write as _;

    let log_path = root
        .join(".ctx")
        .join("godmode")
        .join("traces")
        .join("hooks.log");
    if let Some(parent) = log_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let ts = chrono::Local::now().to_rfc3339();
    let stderr_snippet: String = stderr.chars().take(120).collect();

    // Escape the fields manually to avoid pulling in serde_json here — the
    // strings are all controlled by us so simple JSON escaping suffices.
    let line = format!(
        "{{\"hook\":{},\"event\":{},\"exit_code\":{},\"stderr\":{},\"ts\":{}}}\n",
        json_str(hook),
        json_str(event),
        exit_code,
        json_str(&stderr_snippet),
        json_str(&ts),
    );

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;
    file.write_all(line.as_bytes())
}

/// Minimal JSON string encoder (escapes `\`, `"`, and control chars).
fn json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Read up to `tail` lines from `.ctx/godmode/traces/hooks.log`.
/// Returns an empty vec if the file does not exist.
pub fn read_hook_log(root: &Path, tail: usize) -> std::io::Result<Vec<String>> {
    let log_path = root
        .join(".ctx")
        .join("godmode")
        .join("traces")
        .join("hooks.log");
    if !log_path.exists() {
        return Ok(vec![]);
    }
    let content = std::fs::read_to_string(&log_path)?;
    let lines: Vec<String> = content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.to_string())
        .collect();
    let start = lines.len().saturating_sub(tail);
    Ok(lines[start..].to_vec())
}

/// Parse a hooks.json structure into a flat list of hook entries for display.
/// Returns `(event, matcher, script)` tuples.
pub fn list_hooks_from_json(hooks_json: &serde_json::Value) -> Vec<(String, String, String)> {
    let mut entries = Vec::new();
    let Some(hooks_map) = hooks_json.get("hooks").and_then(|v| v.as_object()) else {
        return entries;
    };
    for (event, matchers) in hooks_map {
        let Some(matchers_arr) = matchers.as_array() else {
            continue;
        };
        for matcher_obj in matchers_arr {
            let matcher = matcher_obj
                .get("matcher")
                .and_then(|v| v.as_str())
                .unwrap_or("*")
                .to_string();
            let Some(hooks_arr) = matcher_obj.get("hooks").and_then(|v| v.as_array()) else {
                continue;
            };
            for hook in hooks_arr {
                let script = hook
                    .get("command")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                entries.push((event.clone(), matcher.clone(), script));
            }
        }
    }
    entries
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn append_and_read_hook_event() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        append_hook_event(root, "pre-bash-nag.nu", "PreToolUse/Bash", 0, "").unwrap();
        append_hook_event(root, "stop-guard.nu", "Stop", 1, "blocked reason").unwrap();
        let lines = read_hook_log(root, 20).unwrap();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("pre-bash-nag.nu"));
        assert!(lines[1].contains("stop-guard.nu"));
        assert!(lines[1].contains("\"exit_code\":1"));
    }

    #[test]
    fn read_hook_log_tail() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        for i in 0..5 {
            append_hook_event(root, &format!("hook{i}"), "Stop", 0, "").unwrap();
        }
        let lines = read_hook_log(root, 3).unwrap();
        assert_eq!(lines.len(), 3);
        assert!(lines[0].contains("hook2"));
    }

    #[test]
    fn read_hook_log_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        let lines = read_hook_log(dir.path(), 20).unwrap();
        assert!(lines.is_empty());
    }

    #[test]
    fn list_hooks_from_json_parses_hooks() {
        let json = serde_json::json!({
            "hooks": {
                "PreToolUse": [{
                    "matcher": "Bash",
                    "hooks": [{"type":"command","command":"nu pre-bash.nu","timeout":10}]
                }]
            }
        });
        let entries = list_hooks_from_json(&json);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, "PreToolUse");
        assert_eq!(entries[0].1, "Bash");
        assert!(entries[0].2.contains("pre-bash.nu"));
    }

    #[test]
    fn stderr_truncated_to_120_chars() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let long_stderr = "x".repeat(200);
        append_hook_event(root, "h", "Stop", 0, &long_stderr).unwrap();
        let lines = read_hook_log(root, 1).unwrap();
        // The JSON-encoded stderr field should not exceed 120 chars of content
        let line = &lines[0];
        // Find the stderr value between quotes after "stderr":
        let stderr_val: String = "x".repeat(120);
        assert!(line.contains(&stderr_val));
        assert!(!line.contains(&"x".repeat(121)));
    }
}
