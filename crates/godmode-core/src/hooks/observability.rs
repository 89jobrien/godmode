//! observability-as-infrastructure — PostToolUse/Bash hook.
//! Appends a JSONL trace event when godmode state-transition commands are detected.

use std::path::Path;

use serde_json::json;

use super::trace_log;

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

    let cmd_short = if command.len() > 80 {
        &command[..80]
    } else {
        command
    };

    trace_log::append(
        root,
        "hook_observed",
        json!({"cmd": cmd_short, "exit_code": exit_code}),
    );

    // Silent hook — no output
    String::new()
}
