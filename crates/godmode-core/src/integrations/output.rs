//! Output types for session boundary commands (handon / handoff).

use serde::Serialize;

/// Output from `godmode handon` — suitable for both human and JSON consumers.
#[derive(Debug, Serialize)]
pub struct HandonOutput {
    /// Human-readable text (pre-formatted).
    pub human: String,
    /// Session summary from the local task graph.
    pub graph: GraphOut,
    /// Next todo from doob (raw JSON value), if doob is available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_todo: Option<serde_json::Value>,
    /// hj handon output, if hj is available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hj: Option<String>,
}

/// Output from `godmode handoff`.
#[derive(Debug, Serialize)]
pub struct HandoffOutput {
    pub human: String,
    pub graph: GraphOut,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hj: Option<String>,
    /// Untracked or modified files in the working tree.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub dirty_files: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct GraphOut {
    pub done: usize,
    pub running: usize,
    pub pending: usize,
    pub blocked: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub running_tasks: Vec<String>,
}
