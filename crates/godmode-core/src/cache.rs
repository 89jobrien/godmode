// cache.rs -- writes ~/.cache/godmode/status.json after every graph save.
//
// Downstream consumers (Starship, Nu hooks) read this file to display
// task counts without shelling out to `godmode status`. Cache writes are
// always best-effort: a failure must never fail the CLI command.

use std::fs;
use std::path::PathBuf;

use chrono::Utc;
use serde::Serialize;

use crate::model::{Status, TaskGraph};

#[derive(Serialize)]
pub struct StatusCache {
    pub updated_at: String,
    pub project: String,
    pub pending: usize,
    pub running: usize,
    pub blocked: usize,
    pub done: usize,
}

pub fn cache_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    PathBuf::from(home).join(".cache/godmode/status.json")
}

pub fn build_cache(graph: &TaskGraph, project: &str) -> StatusCache {
    let mut pending = 0;
    let mut running = 0;
    let mut blocked = 0;
    let mut done = 0;
    for t in &graph.tasks {
        match t.status {
            Status::Pending => pending += 1,
            Status::Running => running += 1,
            Status::Blocked => blocked += 1,
            Status::Done => done += 1,
        }
    }
    StatusCache {
        updated_at: Utc::now().to_rfc3339(),
        project: project.to_string(),
        pending,
        running,
        blocked,
        done,
    }
}

pub fn write_status_cache(graph: &TaskGraph, project: &str) {
    let cache = build_cache(graph, project);
    let path = cache_path();
    let Ok(json) = serde_json::to_string_pretty(&cache) else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(&path, json);
}
