//! Shared JSONL trace-event writer for hooks.
//!
//! Appends events to `.ctx/godmode/traces/trace.jsonl`, the file
//! `trace_query` reads. Centralised here because `observability`,
//! `agent_governance`, and `parallel_agents` all need to emit to the same
//! file with the same session-id lookup.

use std::path::Path;

use chrono::Utc;
use crux_runtime::types::step::{Step, StepKind, StepStatus};
use serde_json::{Value, json};

use crate::trace_store;

/// Repository root of the current working directory, if there is one.
///
/// Re-exported from [`trace_store`], which owns all filesystem and subprocess
/// access, so that callers have one import for the whole trace stack.
pub use crate::trace_store::discover_root;

/// Append one event in the **legacy** `{event, session_id, ts, ...}` shape.
///
/// Retained for hooks that have not moved to the crux-typed helpers yet. New
/// callers should prefer [`emit`], which writes a real [`Step`]. The reader
/// accepts both shapes so logs written across the transition stay queryable.
pub fn append(root: &Path, event_name: &str, fields: Value) {
    let mut event = json!({
        "event": event_name,
        "session_id": read_session_id(root),
        "ts": Utc::now().to_rfc3339(),
    });
    if let (Value::Object(base), Value::Object(extra)) = (&mut event, fields) {
        base.extend(extra);
    }
    if let Ok(line) = serde_json::to_string(&event) {
        trace_store::append_line(root, &line);
    }
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

// ---------------------------------------------------------------------------
// crux-typed emission
// ---------------------------------------------------------------------------

/// Which half of a lifecycle a line represents.
///
/// crux's [`Step`] models a *resolved* step, but a JSONL log records both halves
/// of a lifecycle as separate lines, and the unresolved-agent query depends on
/// seeing the open half. The discriminator therefore lives in
/// [`Step::metadata`] under `phase`, which is the extension point crux provides
/// for exactly this kind of per-writer field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Start,
    Terminal,
}

impl Phase {
    pub fn as_str(self) -> &'static str {
        match self {
            Phase::Start => "start",
            Phase::Terminal => "terminal",
        }
    }
}

/// A trace event to append, described in crux's terms.
#[derive(Debug, Clone)]
pub struct NewEvent {
    /// Actor identity: skill name, agent id, or tool name.
    pub name: String,
    pub kind: StepKind,
    pub phase: Phase,
    /// `None` while the lifecycle is still open.
    pub status: Option<StepStatus>,
    pub duration_ms: u64,
    /// Correlates a start with its terminal record.
    pub trace_id: Option<String>,
    pub error: Option<String>,
    /// Domain-specific fields, merged into `Step::metadata`.
    pub extra: Vec<(String, Value)>,
}

impl NewEvent {
    pub fn new(name: &str, kind: StepKind, phase: Phase) -> Self {
        Self {
            name: name.to_string(),
            kind,
            phase,
            status: None,
            duration_ms: 0,
            trace_id: None,
            error: None,
            extra: Vec::new(),
        }
    }

    pub fn status(mut self, status: StepStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn duration_ms(mut self, ms: u64) -> Self {
        self.duration_ms = ms;
        self
    }

    pub fn trace_id(mut self, tid: &str) -> Self {
        self.trace_id = Some(tid.to_string());
        self
    }

    pub fn error(mut self, message: &str) -> Self {
        self.error = Some(message.to_string());
        self
    }

    pub fn with(mut self, key: &str, value: Value) -> Self {
        self.extra.push((key.to_string(), value));
        self
    }
}

/// Build a trace [`Step`] from `ev`, stamped with the current session id.
///
/// Pure: the caller decides where the result goes. On an open step crux has no
/// "pending" status, so `Ok` is written as a schema formality; the reader treats
/// `phase` as authoritative and reports an open step as having no outcome.
pub fn to_step(ev: NewEvent, session_id: &str) -> Step {
    let mut metadata = std::collections::HashMap::new();
    metadata.insert("phase".to_string(), Value::String(ev.phase.as_str().into()));
    metadata.insert("session_id".to_string(), Value::String(session_id.into()));
    if let Some(tid) = &ev.trace_id {
        metadata.insert("trace_id".to_string(), Value::String(tid.clone()));
    }
    for (key, value) in ev.extra {
        metadata.insert(key, value);
    }

    Step {
        name: ev.name,
        kind: ev.kind,
        status: ev.status.unwrap_or(StepStatus::Ok),
        confidence: 1.0,
        started_at: Utc::now(),
        duration_ms: ev.duration_ms,
        input_hash: 0,
        content_hash: None,
        output: None,
        error: ev.error,
        attempt: 1,
        events: Vec::new(),
        metadata,
        findings: Vec::new(),
    }
}

/// Append one event to the log. Best-effort: never fails the caller.
pub fn emit(root: &Path, ev: NewEvent) {
    let step = to_step(ev, &read_session_id(root));
    let _ = serde_json::to_string(&step).map(|line| trace_store::append_line(root, &line));
}

/// Mint a trace id correlating a skill's start with its terminal record.
///
/// The counter matters: two helpers with the same skill and helper that start in
/// the same millisecond would otherwise share an id, and their start/terminal
/// pairs would interleave into an unresolvable tangle. The counter is
/// process-local; it is seeded from the clock so ids stay ordered within a run.
pub fn new_trace_id(skill: &str, helper: &str) -> String {
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let millis = Utc::now().timestamp_millis().max(0) as u64;
    format!("{skill}.{helper}#{millis}.{n}")
}

/// Open a skill lifecycle. Returns the trace id for the terminal call.
pub fn skill_start(root: &Path, skill: &str, helper: &str, args: &[String]) -> String {
    let tid = new_trace_id(skill, helper);
    let argv = Value::Array(args.iter().map(|a| Value::String(a.clone())).collect());
    emit(
        root,
        NewEvent::new(skill, StepKind::Plain, Phase::Start)
            .trace_id(&tid)
            .with("helper", Value::String(helper.into()))
            .with("args", argv),
    );
    tid
}

/// Close a skill lifecycle successfully.
pub fn skill_complete(root: &Path, skill: &str, trace_id: &str, duration_ms: u64) {
    emit(
        root,
        NewEvent::new(skill, StepKind::Plain, Phase::Terminal)
            .status(StepStatus::Ok)
            .duration_ms(duration_ms)
            .trace_id(trace_id),
    );
}

/// Close a skill lifecycle with a failure.
pub fn skill_error(
    root: &Path,
    skill: &str,
    trace_id: &str,
    exit_code: i64,
    stderr_tail: &str,
    duration_ms: u64,
) {
    emit(
        root,
        NewEvent::new(skill, StepKind::Plain, Phase::Terminal)
            .status(StepStatus::Err)
            .duration_ms(duration_ms)
            .trace_id(trace_id)
            .error(&format!("exit {exit_code}"))
            .with("exit_code", Value::from(exit_code))
            .with("stderr_tail", Value::String(stderr_tail.into())),
    );
}

/// Record a branching decision, e.g. a CI classification.
pub fn decision(root: &Path, skill: &str, helper: &str, kind: &str, value: &str) {
    emit(
        root,
        NewEvent::new(skill, StepKind::Branch, Phase::Terminal)
            .status(StepStatus::Ok)
            .with("helper", Value::String(helper.into()))
            .with("decision_kind", Value::String(kind.into()))
            .with("decision_value", Value::String(value.into())),
    );
}

/// Record a tool invocation.
pub fn tool_use(root: &Path, tool: &str, failed: bool) {
    let status = if failed {
        StepStatus::Err
    } else {
        StepStatus::Ok
    };
    emit(
        root,
        NewEvent::new(tool, StepKind::Plain, Phase::Terminal)
            .status(status)
            .with("tool", Value::String(tool.into())),
    );
}

/// Open an agent lifecycle. `agent.start` is the record that can go unresolved.
pub fn agent_start(root: &Path, agent_id: &str, slot: &str, crate_name: &str) {
    emit(
        root,
        NewEvent::new(agent_id, StepKind::Delegation, Phase::Start)
            .with("slot", Value::String(slot.into()))
            .with("crate", Value::String(crate_name.into())),
    );
}

/// Close an agent lifecycle successfully.
pub fn agent_complete(root: &Path, agent_id: &str, slot: &str, commits: &[String]) {
    let list = Value::Array(commits.iter().map(|c| Value::String(c.clone())).collect());
    emit(
        root,
        NewEvent::new(agent_id, StepKind::Delegation, Phase::Terminal)
            .status(StepStatus::Ok)
            .with("slot", Value::String(slot.into()))
            .with("commits", list),
    );
}

/// Close an agent lifecycle as blocked.
///
/// `status` is `Rejected`, not `Err`: a blocked agent is a policy decision, not
/// a failure, and collapsing the two would make `trace failures` report every
/// gate rejection as an error.
pub fn agent_blocked(root: &Path, agent_id: &str, slot: &str, reason: &str) {
    emit(
        root,
        NewEvent::new(agent_id, StepKind::Delegation, Phase::Terminal)
            .status(StepStatus::Rejected)
            .error(reason)
            .with("slot", Value::String(slot.into())),
    );
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

    // --- crux-typed emission, round-tripped through the reader -----------
    //
    // The point of moving emission into this module is that the writer and the
    // reader cannot disagree. These tests emit through the public API and read
    // back with `trace_query`, so a divergence fails here rather than in a
    // helper script three repos away.

    /// Read the log back the way `godmode trace` does.
    fn read_back(dir: &TempDir) -> Vec<crate::trace_query::Record> {
        {
            let body = crate::trace_store::read_body(dir.path()).unwrap_or_default();
            crate::trace_query::parse(&body)
        }
    }

    /// Metadata holds arbitrary JSON, so numeric and array values are compared
    /// as values rather than through `Record::meta_str`, which is string-only.
    fn meta(r: &crate::trace_query::Record, key: &str) -> String {
        r.metadata
            .get(key)
            .map_or_else(String::new, Value::to_string)
    }

    #[test]
    fn skill_lifecycle_round_trips_as_open_then_terminal() {
        let dir = TempDir::new().unwrap();
        let tid = skill_start(dir.path(), "ci-fix", "fetch-failure.nu", &["--json".into()]);
        skill_complete(dir.path(), "ci-fix", &tid, 120);

        let records = read_back(&dir);
        assert_eq!(records.len(), 2);

        let open = &records[0];
        assert_eq!(open.name, "ci-fix");
        assert!(open.is_open());
        assert_eq!(open.status, None, "an open skill has no outcome yet");
        assert_eq!(open.trace_id.as_deref(), Some(tid.as_str()));

        let done = &records[1];
        assert!(!done.is_open());
        assert_eq!(done.status, Some(StepStatus::Ok));
        assert_eq!(done.duration_ms, Some(120));
        assert_eq!(done.trace_id.as_deref(), Some(tid.as_str()));
    }

    #[test]
    fn skill_start_carries_helper_and_args() {
        let dir = TempDir::new().unwrap();
        skill_start(dir.path(), "cap", "cap.nu", &["--dry".into(), "x".into()]);
        let records = read_back(&dir);
        assert_eq!(records[0].meta_str("helper"), Some("cap.nu"));
        assert_eq!(meta(&records[0], "args"), r#"["--dry","x"]"#);
    }

    #[test]
    fn skill_error_round_trips_as_a_failure_with_its_exit_code() {
        let dir = TempDir::new().unwrap();
        let tid = skill_start(dir.path(), "cron-refresh", "c.nu", &[]);
        skill_error(dir.path(), "cron-refresh", &tid, 3, "boom", 42);

        let records = read_back(&dir);
        let failed = &records[1];
        assert!(failed.is_failure());
        assert_eq!(failed.status, Some(StepStatus::Err));
        assert_eq!(failed.duration_ms, Some(42));
        assert_eq!(meta(failed, "exit_code"), "3");
        assert_eq!(failed.meta_str("stderr_tail"), Some("boom"));
    }

    #[test]
    fn an_agent_start_with_no_terminal_is_reported_unresolved() {
        // The exact shape of the six `subagent-explore` records in the live
        // trace: dispatched, never reported back.
        let dir = TempDir::new().unwrap();
        agent_start(dir.path(), "subagent-explore", "1", "godmode-core");

        let records = read_back(&dir);
        assert!(records[0].is_agent());
        assert!(records[0].is_open());

        let refs: Vec<&crate::trace_query::Record> = records.iter().collect();
        assert_eq!(
            crate::trace_query::unresolved_agents(&refs),
            vec!["subagent-explore".to_string()]
        );
    }

    #[test]
    fn agent_start_carries_slot_and_crate() {
        // `slot` is documented on the blocked record; emitting it on the open
        // record too means a reader never has to treat it as possibly-absent.
        let dir = TempDir::new().unwrap();
        agent_start(dir.path(), "a", "3", "minibox");
        let records = read_back(&dir);
        assert_eq!(records[0].meta_str("slot"), Some("3"));
        assert_eq!(records[0].meta_str("crate"), Some("minibox"));
    }

    #[test]
    fn agent_blocked_round_trips_as_rejected_not_err() {
        let dir = TempDir::new().unwrap();
        agent_start(dir.path(), "a", "1", "c");
        agent_blocked(dir.path(), "a", "1", "policy denied");

        let records = read_back(&dir);
        let blocked = &records[1];
        assert_eq!(blocked.status, Some(StepStatus::Rejected));
        assert!(blocked.is_failure());
        assert_eq!(blocked.error.as_deref(), Some("policy denied"));

        let refs: Vec<&crate::trace_query::Record> = records.iter().collect();
        assert!(
            crate::trace_query::unresolved_agents(&refs).is_empty(),
            "a blocked agent is closed, not unresolved"
        );
    }

    #[test]
    fn agent_complete_resolves_the_open_record() {
        let dir = TempDir::new().unwrap();
        agent_start(dir.path(), "a", "1", "c");
        agent_complete(dir.path(), "a", "1", &["abc123".to_string()]);

        let records = read_back(&dir);
        let refs: Vec<&crate::trace_query::Record> = records.iter().collect();
        assert!(crate::trace_query::unresolved_agents(&refs).is_empty());
        assert_eq!(meta(&records[1], "commits"), r#"["abc123"]"#);
    }

    #[test]
    fn decision_round_trips_as_a_branch() {
        let dir = TempDir::new().unwrap();
        decision(
            dir.path(),
            "ci-fix",
            "fetch-failure.nu",
            "ci_class",
            "test_failure",
        );

        let records = read_back(&dir);
        assert!(records[0].is_decision());
        assert_eq!(records[0].meta_str("decision_kind"), Some("ci_class"));
        assert_eq!(records[0].meta_str("decision_value"), Some("test_failure"));
    }

    #[test]
    fn tool_use_failure_round_trips_as_err() {
        let dir = TempDir::new().unwrap();
        tool_use(dir.path(), "Bash", true);
        tool_use(dir.path(), "Read", false);

        let records = read_back(&dir);
        assert!(records[0].is_failure());
        assert!(!records[1].is_failure());
        assert_eq!(records[0].meta_str("tool"), Some("Bash"));
    }

    #[test]
    fn emitted_lines_are_crux_steps_on_disk() {
        // The on-disk form must be crux's schema, not a godmode-private one,
        // or the "owned by crux" claim is only true in memory.
        let dir = TempDir::new().unwrap();
        agent_start(dir.path(), "a", "1", "c");
        let raw =
            std::fs::read_to_string(dir.path().join(".ctx/godmode/traces/trace.jsonl")).unwrap();
        let line: Value = serde_json::from_str(raw.lines().next().unwrap()).unwrap();

        assert!(line.get("name").is_some());
        assert_eq!(line.get("kind").and_then(Value::as_str), Some("delegation"));
        assert_eq!(line.get("status").and_then(Value::as_str), Some("ok"));
        assert!(line.get("started_at").is_some());
        assert_eq!(
            line.pointer("/metadata/phase").and_then(Value::as_str),
            Some("start")
        );
    }

    #[test]
    fn emission_is_best_effort_and_never_panics() {
        // A trace write must never be able to fail the caller. A path that
        // cannot be created stands in for a read-only or invalid root.
        let blocked = tempfile::TempDir::new().unwrap();
        let file = blocked.path().join("not-a-dir");
        std::fs::write(&file, b"x").unwrap();
        agent_start(&file, "a", "1", "c");
        skill_error(&file, "s", "t", 1, "e", 0);
    }

    #[test]
    fn new_trace_id_correlates_and_is_distinct() {
        let a = new_trace_id("skill", "helper.nu");
        let b = new_trace_id("skill", "helper.nu");
        assert!(a.starts_with("skill.helper.nu#"));
        assert_ne!(a, b, "ids must be unique per call");
    }
}
