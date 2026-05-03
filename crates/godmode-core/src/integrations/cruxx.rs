//! Cruxx trace integration — emit JSONL task events to `.ctx/GODMODE.trace.jsonl`.
//!
//! Events are append-only JSONL, schema-compatible with the slashcrux vocabulary.
//! Each line is a JSON object using `slashcrux::StepState` for the `state` field,
//! keyed by `step_name` (task ID) and enriched with godmode-specific metadata.

use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use slashcrux::StepState;

fn now_rfc3339() -> String {
    Utc::now().to_rfc3339()
}

/// A godmode task event, schema-compatible with the slashcrux step vocabulary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskEvent {
    /// The task ID (e.g. `t1`).
    pub step_name: String,
    /// Human-readable task title.
    pub title: String,
    /// Lifecycle state from the slashcrux vocabulary.
    pub state: StepState,
    /// RFC 3339 timestamp — set at construction time, always present in serialised output.
    #[serde(default = "now_rfc3339")]
    pub ts: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crate_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl TaskEvent {
    pub fn pending(task_id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            step_name: task_id.into(),
            title: title.into(),
            state: StepState::Pending,
            ts: now_rfc3339(),
            crate_name: None,
            commit: None,
            notes: None,
            reason: None,
        }
    }

    pub fn started(task_id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            step_name: task_id.into(),
            title: title.into(),
            state: StepState::Running,
            ts: now_rfc3339(),
            crate_name: None,
            commit: None,
            notes: None,
            reason: None,
        }
    }

    pub fn completed(
        task_id: impl Into<String>,
        title: impl Into<String>,
        commit: Option<String>,
        notes: Option<String>,
    ) -> Self {
        Self {
            step_name: task_id.into(),
            title: title.into(),
            state: StepState::Completed,
            ts: now_rfc3339(),
            crate_name: None,
            commit,
            notes,
            reason: None,
        }
    }

    /// A task that is externally blocked (waiting on something outside godmode).
    ///
    /// Uses `StepState::Cancelled` — not `Failed` — because the task was not
    /// attempted and found wanting; it was externally stopped before it could run.
    pub fn blocked(
        task_id: impl Into<String>,
        title: impl Into<String>,
        reason: Option<String>,
    ) -> Self {
        Self {
            step_name: task_id.into(),
            title: title.into(),
            state: StepState::Cancelled,
            ts: now_rfc3339(),
            crate_name: None,
            commit: None,
            notes: None,
            reason,
        }
    }
}

/// Resolve the trace file path: `<root>/.ctx/GODMODE.trace.jsonl`.
pub fn trace_file(root: &Path) -> PathBuf {
    root.join(".ctx").join("GODMODE.trace.jsonl")
}

/// Append one event as a JSONL line. Creates `.ctx/` and the file if needed.
///
/// The `ts` field is already embedded in the event struct — no post-serialisation
/// mutation is needed.
pub fn append_event(root: &Path, event: &TaskEvent) -> Result<()> {
    let path = trace_file(root);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let line = serde_json::to_string(event).context("serialise JSONL line")?;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .with_context(|| format!("opening {}", path.display()))?;
    writeln!(file, "{}", line).with_context(|| format!("writing {}", path.display()))
}
