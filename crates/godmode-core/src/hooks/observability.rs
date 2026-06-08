//! observability-as-infrastructure — PostToolUse/Bash hook.
//! Appends a JSONL trace event when godmode state-transition commands are detected.

use std::path::Path;

use chrono::Utc;
use serde_json::json;

const WATCHED: &[&str] = &[
    "godmode task start",
    "godmode task done",
    "godmode wave",
    "godmode worktree",
];

/// Run the observability hook. Returns a message for stderr (always empty — silent hook).
pub fn run(root: &Path, command: &str, exit_code: i64) -> String {
    if !WATCHED.iter().any(|pat| command.contains(pat)) {
        return String::new();
    }

    let trace_path = root.join(".ctx/godmode/traces/trace.jsonl");
    if let Some(parent) = trace_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    // Read session ID
    let session_file = root.join(".ctx/godmode/session.json");
    let session_id = std::fs::read_to_string(&session_file)
        .ok()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        .and_then(|v| {
            v.get("session_id")
                .and_then(|s| s.as_str())
                .map(String::from)
        })
        .unwrap_or_default();

    let cmd_short = if command.len() > 80 {
        &command[..80]
    } else {
        command
    };

    let event = json!({
        "event": "hook_observed",
        "cmd": cmd_short,
        "session_id": session_id,
        "ts": Utc::now().to_rfc3339(),
        "exit_code": exit_code,
    });

    let line = format!("{}\n", event);
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&trace_path)
        .and_then(|mut f| std::io::Write::write_all(&mut f, line.as_bytes()));

    // Silent hook — no output
    String::new()
}
