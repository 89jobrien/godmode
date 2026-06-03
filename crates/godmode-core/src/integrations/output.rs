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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handon_output_serializes_skipping_none() {
        let out = HandonOutput {
            human: "test".into(),
            graph: GraphOut {
                done: 1,
                running: 0,
                pending: 2,
                blocked: 0,
                running_tasks: vec![],
            },
            next_todo: None,
            hj: None,
        };
        let json = serde_json::to_string(&out).unwrap();
        assert!(!json.contains("next_todo"), "None fields should be skipped");
        assert!(!json.contains("hj"), "None fields should be skipped");
        assert!(
            !json.contains("running_tasks"),
            "empty vec should be skipped"
        );
    }

    #[test]
    fn handoff_output_includes_dirty_files() {
        let out = HandoffOutput {
            human: "done".into(),
            graph: GraphOut {
                done: 0,
                running: 0,
                pending: 0,
                blocked: 0,
                running_tasks: vec![],
            },
            hj: None,
            dirty_files: vec!["M src/lib.rs".into()],
        };
        let json = serde_json::to_string(&out).unwrap();
        assert!(json.contains("dirty_files"));
        assert!(json.contains("src/lib.rs"));
    }

    #[test]
    fn graph_out_empty_running_tasks_skipped() {
        let g = GraphOut {
            done: 5,
            running: 0,
            pending: 0,
            blocked: 0,
            running_tasks: vec![],
        };
        let json = serde_json::to_string(&g).unwrap();
        assert!(!json.contains("running_tasks"));
    }

    #[test]
    fn handoff_output_empty_dirty_files_skipped() {
        let out = HandoffOutput {
            human: "".into(),
            graph: GraphOut {
                done: 0,
                running: 0,
                pending: 0,
                blocked: 0,
                running_tasks: vec![],
            },
            hj: None,
            dirty_files: vec![],
        };
        let json = serde_json::to_string(&out).unwrap();
        assert!(!json.contains("dirty_files"));
    }
}
