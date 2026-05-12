use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::model::TaskGraph;

/// Cached status summary written to `~/.cache/godmode/status.json`.
/// Designed for fast reads by starship prompt modules.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StatusCache {
    pub updated_at: DateTime<Utc>,
    pub pending: usize,
    pub running: usize,
    pub blocked: usize,
    pub done: usize,
    pub project: String,
}

/// Default cache directory: `~/.cache/godmode/`.
fn cache_dir() -> Option<PathBuf> {
    dirs_or_home().map(|d| d.join("godmode"))
}

fn dirs_or_home() -> Option<PathBuf> {
    std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))
}

/// Build a `StatusCache` from a `TaskGraph` snapshot.
pub fn build_status_cache(graph: &TaskGraph, project: &str) -> StatusCache {
    let s = graph.summary();
    StatusCache {
        updated_at: Utc::now(),
        pending: s.pending,
        running: s.running,
        blocked: s.blocked,
        done: s.done,
        project: project.to_string(),
    }
}

/// Write the status cache to disk. Best-effort: returns `Ok(())` on success,
/// `Err` on failure. Callers should ignore errors.
pub fn write_status_cache(graph: &TaskGraph, project: &str) -> std::io::Result<()> {
    write_status_cache_to(graph, project, None)
}

/// Write status cache, optionally overriding the target directory (for testing).
pub fn write_status_cache_to(
    graph: &TaskGraph,
    project: &str,
    dir_override: Option<&Path>,
) -> std::io::Result<()> {
    let dir = match dir_override {
        Some(d) => d.to_path_buf(),
        None => cache_dir().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "no cache directory found")
        })?,
    };
    std::fs::create_dir_all(&dir)?;
    let cache = build_status_cache(graph, project);
    let json = serde_json::to_string_pretty(&cache).map_err(|e| std::io::Error::other(e))?;
    std::fs::write(dir.join("status.json"), json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Status, Task, TaskGraph};

    fn sample_graph() -> TaskGraph {
        let mut g = TaskGraph::default();
        g.tasks.push(Task::new("t1", "A"));
        let mut t2 = Task::new("t2", "B");
        t2.status = Status::Running;
        g.tasks.push(t2);
        let mut t3 = Task::new("t3", "C");
        t3.status = Status::Done;
        g.tasks.push(t3);
        let mut t4 = Task::new("t4", "D");
        t4.status = Status::Blocked;
        g.tasks.push(t4);
        let mut t5 = Task::new("t5", "E");
        t5.status = Status::Done;
        g.tasks.push(t5);
        g
    }

    #[test]
    fn cache_counts_match_graph_summary() {
        let g = sample_graph();
        let cache = build_status_cache(&g, "testproject");
        assert_eq!(cache.pending, 1);
        assert_eq!(cache.running, 1);
        assert_eq!(cache.blocked, 1);
        assert_eq!(cache.done, 2);
        assert_eq!(cache.project, "testproject");
    }

    #[test]
    fn cache_file_written_on_save() {
        let dir = tempfile::tempdir().unwrap();
        let g = sample_graph();
        write_status_cache_to(&g, "myproject", Some(dir.path())).unwrap();
        let path = dir.path().join("status.json");
        assert!(path.exists(), "status.json should exist");
        let content = std::fs::read_to_string(&path).unwrap();
        let parsed: StatusCache = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed.pending, 1);
        assert_eq!(parsed.running, 1);
        assert_eq!(parsed.blocked, 1);
        assert_eq!(parsed.done, 2);
        assert_eq!(parsed.project, "myproject");
    }

    #[test]
    fn cache_creates_missing_directory() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("deep").join("nested");
        let g = TaskGraph::default();
        write_status_cache_to(&g, "proj", Some(&nested)).unwrap();
        assert!(nested.join("status.json").exists());
    }

    #[test]
    fn cache_roundtrips_json() {
        let g = sample_graph();
        let cache = build_status_cache(&g, "rtest");
        let json = serde_json::to_string_pretty(&cache).unwrap();
        let back: StatusCache = serde_json::from_str(&json).unwrap();
        assert_eq!(cache, back);
    }
}
