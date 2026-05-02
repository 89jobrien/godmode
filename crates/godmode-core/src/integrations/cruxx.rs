//! Cruxx trace integration — emit JSONL task events to `.ctx/GODMODE.trace.jsonl`.
//!
//! Events are append-only. Each line is a self-contained JSON object compatible with
//! the cruxx trace schema (subset). No cruxx dependency is required — we write the
//! schema by hand against the stable JSONL format.

use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};

/// The kind of task lifecycle event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EventKind {
    Started,
    Completed,
    Blocked,
}

/// A single task lifecycle event written to the cruxx trace file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskEvent {
    pub kind: EventKind,
    pub task_id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crate_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Resolve the trace file path: `<root>/.ctx/GODMODE.trace.jsonl`.
pub fn trace_file(root: &Path) -> PathBuf {
    root.join(".ctx").join("GODMODE.trace.jsonl")
}

/// Append one event as a JSONL line. Creates `.ctx/` and the file if needed.
pub fn append_event(root: &Path, event: &TaskEvent) -> Result<()> {
    let path = trace_file(root);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let ts = Utc::now().to_rfc3339();
    let mut obj = serde_json::to_value(event).context("serialise event")?;
    obj.as_object_mut()
        .expect("object")
        .insert("ts".into(), serde_json::Value::String(ts));
    let line = serde_json::to_string(&obj).context("serialise JSONL line")?;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .with_context(|| format!("opening {}", path.display()))?;
    writeln!(file, "{}", line).with_context(|| format!("writing {}", path.display()))
}
