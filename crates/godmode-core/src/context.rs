//! Session context assembly for hooks, subagents, and machine-readable output.

use anyhow::Result;
use serde::Serialize;
use std::path::Path;

use crate::config::Config;
use crate::graph;
use crate::integrations::coursers;
use crate::integrations::subprocess;
use crate::model::Status;

/// Full session context for hooks and subagents.
#[derive(Debug, Serialize)]
pub struct SessionContext {
    /// Absolute or caller-provided path to the repository root.
    pub git_root: String,
    /// Detected package or directory name for the project.
    pub project: String,
    /// Tasks currently in the running state.
    pub running: Vec<TaskSummary>,
    /// Number of tasks currently pending.
    pub pending_count: usize,
    /// Tasks currently blocked and their recorded reasons.
    pub blocked: Vec<BlockedSummary>,
    /// Recent commits rendered in one-line form.
    pub recent_commits: Vec<String>,
    /// Number of tasks in the active graph's critical path.
    pub critical_path_depth: usize,
    /// Recent command failures reported by the coursers integration.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub coursers_failures: Vec<coursers::FailingSummary>,
}

/// Compact representation of a running task.
#[derive(Debug, Serialize)]
pub struct TaskSummary {
    /// Stable task identifier.
    pub id: String,
    /// Human-readable task title.
    pub title: String,
    /// Optional crate targeted by the task.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crate_name: Option<String>,
}

/// Compact representation of a blocked task.
#[derive(Debug, Serialize)]
pub struct BlockedSummary {
    /// Stable task identifier.
    pub id: String,
    /// Recorded explanation for why the task is blocked.
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

    let cfg = Config::load(root);
    let coursers_failures = if cfg.integrations.crs {
        coursers::failing_commands(root)
    } else {
        Vec::new()
    };

    Ok(SessionContext {
        git_root,
        project,
        running,
        pending_count,
        blocked,
        recent_commits,
        critical_path_depth,
        coursers_failures,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Task, TaskGraph};

    #[test]
    fn build_empty_graph() {
        let dir = tempfile::TempDir::new().unwrap();
        let ctx = build(dir.path()).unwrap();
        assert!(ctx.running.is_empty());
        assert_eq!(ctx.pending_count, 0);
        assert!(ctx.blocked.is_empty());
        assert_eq!(ctx.critical_path_depth, 0);
    }

    #[test]
    fn build_counts_statuses() {
        let dir = tempfile::TempDir::new().unwrap();
        let mut g = TaskGraph::default();
        let mut t1 = Task::new("t1", "Running task");
        t1.status = Status::Running;
        t1.crate_name = Some("foo".into());
        g.tasks.push(t1);
        g.tasks.push(Task::new("t2", "Pending task"));
        let mut t3 = Task::new("t3", "Blocked task");
        t3.status = Status::Blocked;
        t3.notes = "waiting on review".into();
        g.tasks.push(t3);
        graph::save(dir.path(), &g).unwrap();

        let ctx = build(dir.path()).unwrap();
        assert_eq!(ctx.running.len(), 1);
        assert_eq!(ctx.running[0].id, "t1");
        assert_eq!(ctx.running[0].crate_name.as_deref(), Some("foo"));
        assert_eq!(ctx.pending_count, 1);
        assert_eq!(ctx.blocked.len(), 1);
        assert_eq!(ctx.blocked[0].reason, "waiting on review");
    }

    #[test]
    fn build_serializes_to_json() {
        let dir = tempfile::TempDir::new().unwrap();
        let ctx = build(dir.path()).unwrap();
        let json = serde_json::to_string(&ctx).unwrap();
        assert!(json.contains("\"pending_count\""));
        assert!(json.contains("\"critical_path_depth\""));
    }

    #[test]
    fn git_recent_commits_returns_empty_for_non_git_dir() {
        let dir = tempfile::TempDir::new().unwrap();
        let commits = git_recent_commits(dir.path(), 5);
        assert!(commits.is_empty());
    }
}
