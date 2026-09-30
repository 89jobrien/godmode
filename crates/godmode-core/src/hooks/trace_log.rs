//! Shared JSONL trace-event writer for hooks.
//!
//! Appends events to `.ctx/godmode/traces/trace.jsonl`, the file
//! `trace_query` reads. Centralised here because `observability`,
//! `agent_governance`, and `parallel_agents` all need to emit to the same
//! file with the same session-id lookup.

use std::path::{Path, PathBuf};

use chrono::Utc;
use serde_json::{Value, json};

/// Repository root of the current working directory, if there is one.
///
/// Readers and writers must agree on which log they mean, so both go through
/// here rather than each shelling out to git.
pub fn discover_root() -> Option<PathBuf> {
    let out = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()
        .filter(|out| out.status.success())?;
    let root = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (!root.is_empty()).then(|| PathBuf::from(root))
}

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
    session_id(root).unwrap_or_else(|| "no-session".to_string())
}

/// Resolve the current session id, rotating it when the session is stale.
///
/// The id is stable for one working session and re-minted when either the local
/// calendar day changes or `CLAUDE_SESSION_ID` differs from the one recorded
/// last time. Without rotation the id was pinned forever by the first write, so a
/// session from one day silently absorbed every later day's events and
/// cross-session correlation was meaningless.
///
/// `pinned_root` is owned by `godmode pin` and is preserved across rotation.
/// This must agree with `skills/_lib/trace.nu`, which writes the same file.
fn session_id(root: &Path) -> Option<String> {
    let session_file = root.join(".ctx/godmode/session.json");
    let existing: Value = std::fs::read_to_string(&session_file)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or(Value::Null);

    let stored_id = existing.get("session_id").and_then(Value::as_str);
    let stored_day = existing
        .get("started_at")
        .and_then(Value::as_str)
        .and_then(|s| s.get(..10))
        .unwrap_or_default();
    let stored_claude = existing
        .get("claude_session_id")
        .and_then(Value::as_str)
        .unwrap_or_default();

    let today = Utc::now().format("%Y-%m-%d").to_string();
    let claude_id = std::env::var("CLAUDE_SESSION_ID").unwrap_or_default();

    if let Some(id) =
        stored_id.filter(|id| !id.is_empty() && stored_day == today && stored_claude == claude_id)
    {
        return Some(id.to_string());
    }

    // Rotate. Minting needs a git sha, which is best-effort: a repo with no HEAD
    // still gets a usable id rather than an empty string.
    let head = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(root)
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .filter(|sha| !sha.is_empty())
        .unwrap_or_else(|| "unknown".to_string());

    let fresh = format!("{head}-{}", Utc::now().timestamp_millis());

    let mut rotated = match existing {
        Value::Object(map) => map,
        _ => serde_json::Map::new(),
    };
    rotated.insert("session_id".into(), Value::String(fresh.clone()));
    rotated.insert("started_at".into(), Value::String(Utc::now().to_rfc3339()));
    rotated.insert("claude_session_id".into(), Value::String(claude_id));

    if let Some(parent) = session_file.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(body) = serde_json::to_string(&Value::Object(rotated)) {
        let _ = std::fs::write(&session_file, body);
    }

    Some(fresh)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use tempfile::TempDir;

    fn seed(dir: &TempDir, body: Value) {
        let ctx = dir.path().join(".ctx/godmode");
        std::fs::create_dir_all(&ctx).unwrap();
        std::fs::write(ctx.join("session.json"), body.to_string()).unwrap();
    }

    fn session_json(dir: &TempDir) -> Value {
        let raw = std::fs::read_to_string(dir.path().join(".ctx/godmode/session.json")).unwrap();
        serde_json::from_str(&raw).unwrap()
    }

    fn today() -> String {
        Utc::now().format("%Y-%m-%d").to_string()
    }

    /// Serialised because the env var is process-global, and a parallel test
    /// flipping it would make this flaky.
    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn reuses_id_within_same_session() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        // SAFETY: guarded by ENV_LOCK; no other test reads CLAUDE_SESSION_ID.
        unsafe { std::env::set_var("CLAUDE_SESSION_ID", "claude-A") };

        let dir = TempDir::new().unwrap();
        seed(
            &dir,
            json!({
                "session_id": "keep-me",
                "started_at": format!("{}T00:00:00Z", today()),
                "claude_session_id": "claude-A",
            }),
        );

        assert_eq!(session_id(dir.path()).as_deref(), Some("keep-me"));
    }

    #[test]
    fn rotates_when_the_day_changed() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe { std::env::set_var("CLAUDE_SESSION_ID", "claude-A") };

        let dir = TempDir::new().unwrap();
        seed(
            &dir,
            json!({
                "session_id": "stale",
                "started_at": "2020-01-01T00:00:00Z",
                "claude_session_id": "claude-A",
            }),
        );

        let id = session_id(dir.path()).expect("an id must be minted");
        assert_ne!(id, "stale");
        assert!(id.contains('-'), "expected <sha>-<millis>, got {id}");
    }

    #[test]
    fn rotates_when_claude_session_changed_within_the_day() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe { std::env::set_var("CLAUDE_SESSION_ID", "claude-B") };

        let dir = TempDir::new().unwrap();
        seed(
            &dir,
            json!({
                "session_id": "yesterday-same-day",
                "started_at": format!("{}T00:00:00Z", today()),
                "claude_session_id": "claude-A",
            }),
        );

        assert_ne!(
            session_id(dir.path()).as_deref(),
            Some("yesterday-same-day")
        );
    }

    /// The bug this guards: an id that never persists its rotation inputs fires
    /// on every single call, so no two events ever share a session.
    #[test]
    fn rotation_stabilises_after_the_first_call() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe { std::env::set_var("CLAUDE_SESSION_ID", "claude-C") };

        let dir = TempDir::new().unwrap();
        seed(
            &dir,
            json!({
                "session_id": "stale",
                "started_at": "2020-01-01T00:00:00Z",
                "claude_session_id": "claude-A",
            }),
        );

        let first = session_id(dir.path());
        let second = session_id(dir.path());
        let third = session_id(dir.path());
        assert_eq!(first, second, "rotation must not fire again immediately");
        assert_eq!(second, third, "rotation must not fire again immediately");
    }

    /// `godmode pin` owns this field; rotation must not clobber it.
    #[test]
    fn rotation_preserves_pinned_root() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe { std::env::set_var("CLAUDE_SESSION_ID", "claude-A") };

        let dir = TempDir::new().unwrap();
        seed(
            &dir,
            json!({
                "session_id": "stale",
                "started_at": "2020-01-01T00:00:00Z",
                "pinned_root": "/some/pinned/path",
            }),
        );

        session_id(dir.path());
        assert_eq!(
            session_json(&dir)
                .get("pinned_root")
                .and_then(Value::as_str),
            Some("/some/pinned/path"),
            "pinned_root must survive rotation"
        );
    }

    #[test]
    fn missing_session_file_is_created_rather_than_yielding_empty() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe { std::env::set_var("CLAUDE_SESSION_ID", "") };

        let dir = TempDir::new().unwrap();
        let id = session_id(dir.path()).expect("an id must be minted");
        assert!(
            !id.is_empty(),
            "an empty id breaks cross-session correlation"
        );
        assert!(session_json(&dir).get("session_id").is_some());
    }

    #[test]
    fn append_stamps_a_non_empty_session_id() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe { std::env::set_var("CLAUDE_SESSION_ID", "claude-A") };

        let dir = TempDir::new().unwrap();
        append(dir.path(), "tool.use", json!({ "tool": "Bash" }));

        let raw =
            std::fs::read_to_string(dir.path().join(".ctx/godmode/traces/trace.jsonl")).unwrap();
        let event: Value = serde_json::from_str(raw.lines().next().unwrap()).unwrap();

        let id = event.get("session_id").and_then(Value::as_str).unwrap();
        assert!(
            !id.is_empty(),
            "events must never carry an empty session_id"
        );
        assert_eq!(event.get("event").and_then(Value::as_str), Some("tool.use"));
    }
}
