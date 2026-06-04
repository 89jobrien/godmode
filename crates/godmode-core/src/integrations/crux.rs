//! Cruxx trace integration — build `crux_runtime::Step` values for godmode task transitions.
//!
//! Each constructor returns a `Step` ready to be recorded into a `Session` (see
//! `session_trace`). The `trace_file` path helper is retained for the session layer.

use std::path::{Path, PathBuf};

use chrono::Utc;
use crux_runtime::types::step::{Step, StepKind, StepStatus};
use tracing::instrument;

fn started_at() -> chrono::DateTime<Utc> {
    Utc::now()
}

/// Build a `Step` for a task entering the pending state (just added to the graph).
#[instrument(
    name = "crux::step_pending",
    fields(integration = "crux"),
    skip(task_id)
)]
pub fn step_pending(task_id: impl Into<String>) -> Step {
    Step {
        name: task_id.into(),
        kind: StepKind::Plain,
        status: StepStatus::Skipped, // pending = not yet started
        confidence: 1.0,
        started_at: started_at(),
        duration_ms: 0,
        input_hash: 0,
        content_hash: None,
        output: None,
        error: None,
        attempt: 0,
        events: vec![],
        metadata: Default::default(),
        findings: vec![],
    }
}

/// Build a `Step` for a task transitioning to running.
#[instrument(
    name = "crux::step_started",
    fields(integration = "crux"),
    skip(task_id)
)]
pub fn step_started(task_id: impl Into<String>) -> Step {
    Step {
        name: task_id.into(),
        kind: StepKind::Plain,
        status: StepStatus::Ok,
        confidence: 1.0,
        started_at: started_at(),
        duration_ms: 0,
        input_hash: 0,
        content_hash: None,
        output: None,
        error: None,
        attempt: 1,
        events: vec![],
        metadata: Default::default(),
        findings: vec![],
    }
}

/// Build a `Step` for a task completing successfully.
///
/// `commit` and `notes` are stored as JSON in `output`.
#[instrument(
    name = "crux::step_completed",
    fields(integration = "crux"),
    skip(task_id, commit, notes)
)]
pub fn step_completed(
    task_id: impl Into<String>,
    commit: Option<&str>,
    notes: Option<&str>,
) -> Step {
    let output = if commit.is_some() || notes.is_some() {
        let mut m = serde_json::Map::new();
        if let Some(c) = commit {
            m.insert("commit".into(), serde_json::Value::String(c.into()));
        }
        if let Some(n) = notes {
            m.insert("notes".into(), serde_json::Value::String(n.into()));
        }
        Some(serde_json::Value::Object(m))
    } else {
        None
    };
    Step {
        name: task_id.into(),
        kind: StepKind::Plain,
        status: StepStatus::Ok,
        confidence: 1.0,
        started_at: started_at(),
        duration_ms: 0,
        input_hash: 0,
        content_hash: None,
        output,
        error: None,
        attempt: 1,
        events: vec![],
        metadata: Default::default(),
        findings: vec![],
    }
}

/// Build a `Step` for a task that has been externally blocked.
///
/// Uses `StepStatus::Err` with the reason in `error` — blocked means the task
/// could not proceed due to an external dependency, not an internal failure.
#[instrument(
    name = "crux::step_blocked",
    fields(integration = "crux"),
    skip(task_id, reason)
)]
pub fn step_blocked(task_id: impl Into<String>, reason: Option<&str>) -> Step {
    Step {
        name: task_id.into(),
        kind: StepKind::Plain,
        status: StepStatus::Err,
        confidence: 0.0,
        started_at: started_at(),
        duration_ms: 0,
        input_hash: 0,
        content_hash: None,
        output: None,
        error: reason.map(str::to_string),
        attempt: 1,
        events: vec![],
        metadata: Default::default(),
        findings: vec![],
    }
}

/// Resolve the session directory: `<root>/.ctx/godmode/sessions/`.
pub fn sessions_dir(root: &Path) -> PathBuf {
    root.join(".ctx").join("godmode").join("sessions")
}
