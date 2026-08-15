//! Shared JSONL trace-event writer for hooks.
//!
//! Appends events to `.ctx/godmode/traces/trace.jsonl`, the file
//! `trace-stats.nu` reads. Centralised here because `observability`,
//! `agent_governance`, and `parallel_agents` all need to emit to the same
//! file with the same session-id lookup.

use std::path::Path;

use chrono::Utc;
use serde_json::{Value, json};

/// Append one event to the trace log. `fields` are merged into the event
/// alongside `event`, `session_id`, and `ts`.
pub fn append(root: &Path, event_name: &str, fields: Value) {
    let trace_path = root.join(".ctx/godmode/traces/trace.jsonl");
    if let Some(parent) = trace_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let session_id = read_session_id(root);

    let mut event = json!({
        "event": event_name,
        "session_id": session_id,
        "ts": Utc::now().to_rfc3339(),
    });
    if let (Value::Object(base), Value::Object(extra)) = (&mut event, fields) {
        base.extend(extra);
    }

    let line = format!("{}\n", event);
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&trace_path)
        .and_then(|mut f| std::io::Write::write_all(&mut f, line.as_bytes()));
}

fn read_session_id(root: &Path) -> String {
    let session_file = root.join(".ctx/godmode/session.json");
    std::fs::read_to_string(&session_file)
        .ok()
        .and_then(|s| serde_json::from_str::<Value>(&s).ok())
        .and_then(|v| {
            v.get("session_id")
                .and_then(|s| s.as_str())
                .map(String::from)
        })
        .unwrap_or_default()
}
