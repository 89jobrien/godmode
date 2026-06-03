use anyhow::Result;
use serde::Serialize;
use std::path::Path;

use crate::graph;
use crate::integrations::subprocess;
use crate::model::Status;

/// Full session context for hooks and subagents.
#[derive(Debug, Serialize)]
pub struct SessionContext {
    pub git_root: String,
    pub project: String,
    pub running: Vec<TaskSummary>,
    pub pending_count: usize,
    pub blocked: Vec<BlockedSummary>,
    pub recent_commits: Vec<String>,
    pub critical_path_depth: usize,
}

#[derive(Debug, Serialize)]
pub struct TaskSummary {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crate_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BlockedSummary {
    pub id: String,
    pub reason: String,
}

/// Build the full session context from the repo at `root`.
pub fn build(root: &Path) -> Result<SessionContext> {
    let git_root = root.to_string_lossy().to_string();
    let project = crate::detect::package_name(root).unwrap_or_else(|_| {
        root.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string())
    });
    let task_graph = graph::load(root)?;

    let running: Vec<TaskSummary> = task_graph
        .tasks
        .iter()
        .filter(|t| t.status == Status::Running)
        .map(|t| TaskSummary {
            id: t.id.clone(),
            title: t.title.clone(),
            crate_name: t.crate_name.clone(),
        })
        .collect();

    let pending_count = task_graph
        .tasks
        .iter()
        .filter(|t| t.status == Status::Pending)
        .count();

    let blocked: Vec<BlockedSummary> = task_graph
        .tasks
        .iter()
        .filter(|t| t.status == Status::Blocked)
        .map(|t| BlockedSummary {
            id: t.id.clone(),
            reason: t.notes.clone(),
        })
        .collect();

    let recent_commits = git_recent_commits(root, 5);

    let critical_path_depth = crate::dispatch::critical_path(&task_graph).len();

    Ok(SessionContext {
        git_root,
        project,
        running,
        pending_count,
        blocked,
        recent_commits,
        critical_path_depth,
    })
}

fn git_recent_commits(root: &Path, count: usize) -> Vec<String> {
    subprocess::run_in(
        "git",
        &["log", "--oneline", &format!("-{count}")],
        root,
        "git log for recent commits",
    )
    .map(|out| out.lines().map(str::to_string).collect())
    .unwrap_or_default()
}
