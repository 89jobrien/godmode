use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Task execution status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Pending,
    Running,
    Done,
    Blocked,
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::Pending => write!(f, "pending"),
            Status::Running => write!(f, "running"),
            Status::Done => write!(f, "done"),
            Status::Blocked => write!(f, "blocked"),
        }
    }
}

/// A single task in the execution graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub status: Status,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub notes: String,
    /// Crate targeted by this task, if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crate_name: Option<String>,
    /// Commit SHA recorded on completion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed: Option<NaiveDate>,
}

impl Task {
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            status: Status::Pending,
            depends_on: vec![],
            notes: String::new(),
            crate_name: None,
            commit: None,
            completed: None,
        }
    }
}

/// The full task graph stored in `.ctx/GODMODE.tasks.yaml`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskGraph {
    pub tasks: Vec<Task>,
}

/// Summary counts for display.
#[derive(Debug, Default)]
pub struct GraphSummary {
    pub done: usize,
    pub running: usize,
    pub pending: usize,
    pub blocked: usize,
}

impl TaskGraph {
    pub fn summary(&self) -> GraphSummary {
        let mut s = GraphSummary::default();
        for t in &self.tasks {
            match t.status {
                Status::Done => s.done += 1,
                Status::Running => s.running += 1,
                Status::Pending => s.pending += 1,
                Status::Blocked => s.blocked += 1,
            }
        }
        s
    }
}
