use std::path::{Path, PathBuf};

use anyhow::Result;
use chrono::Utc;
use serde::Serialize;

use crate::config::Config;
use crate::graph;
use crate::integrations::{cruxx, rx};
use crate::model::{GraphSummary, Status, Task, TaskGraph};
use crate::templates;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

pub struct Session {
    root: PathBuf,
    graph: TaskGraph,
    config: Config,
}

#[derive(Debug, Default, Serialize)]
pub struct SessionSummary {
    pub done: usize,
    pub running: usize,
    pub pending: usize,
    pub blocked: usize,
    pub total_duration_ms: u64,
    pub tasks: Vec<TaskTiming>,
}

#[derive(Debug, Serialize)]
pub struct TaskTiming {
    pub id: String,
    pub title: String,
    pub duration_ms: u64,
}

// ---------------------------------------------------------------------------
// Session impl
// ---------------------------------------------------------------------------

impl Session {
    /// Load (or create) a session rooted at `root`, using default config.
    pub fn open(root: &Path) -> Result<Self> {
        Self::open_with_config(root, &Config::load(root))
    }

    /// Load (or create) a session rooted at `root` with explicit config.
    pub fn open_with_config(root: &Path, config: &Config) -> Result<Self> {
        let graph = graph::load(root)?;
        Ok(Self {
            root: root.to_path_buf(),
            graph,
            config: config.clone(),
        })
    }

    pub fn graph(&self) -> &TaskGraph {
        &self.graph
    }

    /// Escape hatch for direct graph mutation. Prefer typed Session methods
    /// (clear_tasks, apply_template) which maintain auto-save and trace invariants.
    #[doc(hidden)]
    pub fn graph_mut(&mut self) -> &mut TaskGraph {
        &mut self.graph
    }

    /// Clear tasks from the graph. Returns the count removed.
    /// `done_only = true` removes only completed tasks; `false` removes all.
    pub fn clear_tasks(&mut self, done_only: bool) -> usize {
        let count = graph::clear(&mut self.graph, done_only);
        if count > 0 {
            self.auto_save();
        }
        count
    }

    /// Apply a template into the task graph. Returns `(applied, skipped)`.
    pub fn apply_template(&mut self, template: templates::Template) -> Result<(usize, usize)> {
        let result = templates::apply(&mut self.graph, template)?;
        if result.0 > 0 {
            self.auto_save();
        }
        Ok(result)
    }

    /// Add a task to the graph.
    pub fn add_task(&mut self, task: Task) -> Result<()> {
        graph::add(&mut self.graph, task)
    }

    /// Remove a task from the graph.
    pub fn remove_task(&mut self, id: &str) -> Result<()> {
        graph::remove(&mut self.graph, id)
    }

    /// Start a task: validate rx script if applicable, set started_at, emit trace Step.
    pub fn start_task(&mut self, id: &str) -> Result<()> {
        if self.config.integrations.rx
            && let Some(task) = self.graph.tasks.iter().find(|t| t.id == id)
            && let Some(run) = &task.run
        {
            rx::validate_run(run)?;
        }
        graph::start(&mut self.graph, id)?;
        if let Some(task) = self.graph.tasks.iter_mut().find(|t| t.id == id) {
            task.started_at = Some(Utc::now());
        }
        if self.config.integrations.cruxx {
            let _ = self.append_step(cruxx::step_started(id));
        }
        self.auto_save();
        Ok(())
    }

    /// Complete a task: compute duration_ms from started_at, emit trace Step.
    pub fn complete_task(
        &mut self,
        id: &str,
        commit: Option<&str>,
        notes: Option<&str>,
    ) -> Result<()> {
        let duration_ms = self
            .graph
            .tasks
            .iter()
            .find(|t| t.id == id)
            .and_then(|t| t.started_at)
            .map(|start| {
                Utc::now()
                    .signed_duration_since(start)
                    .num_milliseconds()
                    .max(0) as u64
            })
            .unwrap_or(0);

        graph::complete(&mut self.graph, id, commit, notes)?;

        if self.config.integrations.cruxx {
            let mut step = cruxx::step_completed(id, commit, notes);
            step.duration_ms = duration_ms;
            if duration_ms < 100 {
                let warn = serde_json::json!({
                    "warn": "suspiciously short duration — was this a real task?"
                });
                step.output = Some(match step.output.take() {
                    Some(serde_json::Value::Object(mut map)) => {
                        map.extend(warn.as_object().unwrap().clone());
                        serde_json::Value::Object(map)
                    }
                    _ => warn,
                });
            }
            let _ = self.append_step(step);
        }
        self.auto_save();
        Ok(())
    }

    /// Block a task with a reason.
    pub fn block_task(&mut self, id: &str, reason: &str) -> Result<()> {
        graph::block(&mut self.graph, id, reason)?;
        if self.config.integrations.cruxx {
            let _ = self.append_step(cruxx::step_blocked(id, Some(reason)));
        }
        self.auto_save();
        Ok(())
    }

    /// Unblock a task.
    pub fn unblock_task(&mut self, id: &str) -> Result<()> {
        graph::unblock(&mut self.graph, id)?;
        self.auto_save();
        Ok(())
    }

    /// Unblock all blocked tasks. Returns count unblocked.
    pub fn unblock_all(&mut self) -> usize {
        let count = graph::unblock_all(&mut self.graph);
        if count > 0 {
            self.auto_save();
        }
        count
    }

    /// Aggregate counts and per-task durations.
    pub fn summary(&self) -> SessionSummary {
        let mut s = SessionSummary::default();
        for task in &self.graph.tasks {
            match task.status {
                Status::Done => s.done += 1,
                Status::Running => s.running += 1,
                Status::Pending => s.pending += 1,
                Status::Blocked => s.blocked += 1,
            }
            let end = task.completed_at.unwrap_or_else(Utc::now);
            let duration_ms = task
                .started_at
                .map(|start| end.signed_duration_since(start).num_milliseconds().max(0) as u64)
                .unwrap_or(0);
            s.tasks.push(TaskTiming {
                id: task.id.clone(),
                title: task.title.clone(),
                duration_ms,
            });
            s.total_duration_ms += duration_ms;
        }
        s
    }

    /// Persist the graph to disk. Caller decides when to call.
    pub fn save(&self) -> Result<()> {
        graph::save(&self.root, &self.graph)
    }

    /// Write a summary record to the summary JSONL file. Non-fatal.
    pub fn write_summary_jsonl(&self, summary: &SessionSummary) -> Result<()> {
        use std::io::Write;
        let dir = cruxx::sessions_dir(&self.root);
        std::fs::create_dir_all(&dir)?;
        let date = chrono::Local::now().format("%Y-%m-%d").to_string();
        let path = dir.join(format!("{date}-summary.jsonl"));
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        writeln!(f, "{}", serde_json::to_string(summary)?)?;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    /// Best-effort flush of graph state to disk after each transition.
    /// Errors are logged but never abort the caller.
    fn auto_save(&self) {
        if let Err(e) = graph::save(&self.root, &self.graph) {
            eprintln!("godmode: auto-save failed: {e}");
        }
    }

    fn append_step(&self, step: cruxx_core::types::step::Step) -> Result<()> {
        use std::io::Write;
        let dir = cruxx::sessions_dir(&self.root);
        std::fs::create_dir_all(&dir)?;
        let date = chrono::Local::now().format("%Y-%m-%d").to_string();
        let path = dir.join(format!("{date}.jsonl"));
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        writeln!(f, "{}", serde_json::to_string(&step)?)?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Backward-compat free functions (thin wrappers)
// ---------------------------------------------------------------------------

/// Print a triage summary to stdout. Called at session start.
pub fn handon(root: &Path) -> Result<()> {
    let s = Session::open(root)?;
    let graph = s.graph();
    if graph.tasks.is_empty() {
        println!("No tasks. Run `godmode plan ingest <plan>` or `godmode task add`.");
        return Ok(());
    }
    let summary = s.summary();
    println!(
        "Tasks: {} done, {} running, {} pending, {} blocked",
        summary.done, summary.running, summary.pending, summary.blocked
    );
    let running: Vec<_> = graph
        .tasks
        .iter()
        .filter(|t| t.status == Status::Running)
        .collect();
    if !running.is_empty() {
        println!("\nIn progress:");
        for t in &running {
            println!("  [{}] {}", t.id, t.title);
        }
    }
    let next = graph::runnable(graph);
    if !next.is_empty() {
        println!("\nNext runnable:");
        for t in &next {
            let tag = t
                .crate_name
                .as_deref()
                .map(|c| format!(" ({})", c))
                .unwrap_or_default();
            let stale_hint = if t.started_at.is_none() {
                " [never started]"
            } else {
                ""
            };
            println!("  [{}] {}{}{}", t.id, t.title, tag, stale_hint);
        }
    }
    let blocked: Vec<_> = graph
        .tasks
        .iter()
        .filter(|t| t.status == Status::Blocked)
        .collect();
    if !blocked.is_empty() {
        println!("\nBlocked:");
        for t in &blocked {
            println!("  [{}] {} — {}", t.id, t.title, t.notes);
        }
    }

    Ok(())
}

/// Validate session end state and emit summary.
pub fn handoff(root: &Path) -> Result<GraphSummary> {
    let s = Session::open(root)?;
    let graph = s.graph();
    let running: Vec<_> = graph
        .tasks
        .iter()
        .filter(|t| t.status == Status::Running)
        .collect();
    if !running.is_empty() {
        eprintln!("Warning: {} task(s) still running:", running.len());
        for t in &running {
            eprintln!("  [{}] {}", t.id, t.title);
        }
        eprintln!("Mark them done or blocked before closing.");
    }
    let session_summary = s.summary();
    println!(
        "\nSession: {} done, {} blocked, total {}ms",
        session_summary.done, session_summary.blocked, session_summary.total_duration_ms
    );
    let _ = s.write_summary_jsonl(&session_summary);
    Ok(graph.summary())
}

// ---------------------------------------------------------------------------
// Session file pruning
// ---------------------------------------------------------------------------

/// Delete session JSONL files in `dir` that are older than `days` days.
/// If `dry_run` is true, prints what would be deleted but makes no changes.
/// Returns the list of paths that were (or would be) deleted.
pub fn prune_sessions_older_than(dir: &Path, days: u64, dry_run: bool) -> Result<Vec<PathBuf>> {
    use std::time::{Duration, SystemTime};

    let cutoff = SystemTime::now()
        .checked_sub(Duration::from_secs(days * 24 * 3600))
        .unwrap_or(SystemTime::UNIX_EPOCH);

    let mut pruned = Vec::new();

    if !dir.exists() {
        return Ok(pruned);
    }

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
            continue;
        }
        let meta = std::fs::metadata(&path)?;
        let modified = meta.modified()?;
        if modified < cutoff {
            if dry_run {
                println!("would delete: {}", path.display());
            } else {
                std::fs::remove_file(&path)?;
            }
            pruned.push(path);
        }
    }

    Ok(pruned)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph;
    use crate::model::Task;
    use tempfile::TempDir;

    #[test]
    fn handon_empty_graph_prints_message() {
        let dir = TempDir::new().unwrap();
        handon(dir.path()).unwrap();
    }

    #[test]
    fn handoff_warns_on_running_tasks() {
        let dir = TempDir::new().unwrap();
        let mut g = crate::model::TaskGraph::default();
        let mut t = Task::new("t1", "Unfinished");
        t.status = Status::Running;
        g.tasks.push(t);
        graph::save(dir.path(), &g).unwrap();
        let summary = handoff(dir.path()).unwrap();
        assert_eq!(summary.running, 1);
    }

    #[test]
    fn session_open_returns_empty_for_missing_file() {
        let dir = TempDir::new().unwrap();
        let s = Session::open(dir.path()).unwrap();
        assert!(s.graph().tasks.is_empty());
    }

    #[test]
    fn session_start_sets_started_at() {
        let dir = TempDir::new().unwrap();
        let mut s = Session::open(dir.path()).unwrap();
        s.add_task(Task::new("t1", "A")).unwrap();
        s.start_task("t1").unwrap();
        let task = s.graph().tasks.iter().find(|t| t.id == "t1").unwrap();
        assert!(task.started_at.is_some());
        assert_eq!(task.status, Status::Running);
    }

    #[test]
    fn session_complete_computes_duration() {
        let dir = TempDir::new().unwrap();
        let mut s = Session::open(dir.path()).unwrap();
        s.add_task(Task::new("t1", "A")).unwrap();
        s.start_task("t1").unwrap();
        s.complete_task("t1", None, None).unwrap();
        let task = s.graph().tasks.iter().find(|t| t.id == "t1").unwrap();
        assert_eq!(task.status, Status::Done);
        let summary = s.summary();
        assert_eq!(summary.done, 1);
        assert_eq!(summary.tasks.len(), 1);
        let _ = summary.tasks[0].duration_ms;
    }

    #[test]
    fn session_save_and_reload() {
        let dir = TempDir::new().unwrap();
        let mut s = Session::open(dir.path()).unwrap();
        s.add_task(Task::new("t1", "A")).unwrap();
        s.save().unwrap();
        let s2 = Session::open(dir.path()).unwrap();
        assert_eq!(s2.graph().tasks.len(), 1);
    }

    #[test]
    fn prune_deletes_old_files_keeps_new() {
        use std::time::{Duration, SystemTime};
        let dir = TempDir::new().unwrap();
        let sessions = dir.path().join("sessions");
        std::fs::create_dir_all(&sessions).unwrap();

        // Write an "old" file and backdate its mtime to 10 days ago.
        let old_file = sessions.join("2020-01-01.jsonl");
        std::fs::write(&old_file, b"old").unwrap();
        let old_time = SystemTime::now()
            .checked_sub(Duration::from_secs(10 * 24 * 3600))
            .unwrap();
        filetime::set_file_mtime(&old_file, filetime::FileTime::from_system_time(old_time))
            .unwrap();

        // Write a "new" file (mtime = now).
        let new_file = sessions.join("2099-01-01.jsonl");
        std::fs::write(&new_file, b"new").unwrap();

        let pruned = prune_sessions_older_than(&sessions, 7, false).unwrap();
        assert_eq!(pruned.len(), 1);
        assert_eq!(pruned[0], old_file);
        assert!(!old_file.exists());
        assert!(new_file.exists());
    }

    #[test]
    fn prune_dry_run_makes_no_changes() {
        use std::time::{Duration, SystemTime};
        let dir = TempDir::new().unwrap();
        let sessions = dir.path().join("sessions");
        std::fs::create_dir_all(&sessions).unwrap();

        let old_file = sessions.join("2020-01-01.jsonl");
        std::fs::write(&old_file, b"old").unwrap();
        let old_time = SystemTime::now()
            .checked_sub(Duration::from_secs(10 * 24 * 3600))
            .unwrap();
        filetime::set_file_mtime(&old_file, filetime::FileTime::from_system_time(old_time))
            .unwrap();

        let pruned = prune_sessions_older_than(&sessions, 7, true).unwrap();
        assert_eq!(pruned.len(), 1);
        // File must still exist after dry-run.
        assert!(old_file.exists());
    }

    #[test]
    fn prune_missing_dir_returns_empty() {
        let dir = TempDir::new().unwrap();
        let sessions = dir.path().join("no-such-dir");
        let pruned = prune_sessions_older_than(&sessions, 7, false).unwrap();
        assert!(pruned.is_empty());
    }

    #[test]
    fn completed_at_set_on_task_done() {
        let dir = TempDir::new().unwrap();
        let mut s = Session::open(dir.path()).unwrap();
        s.add_task(Task::new("t1", "A")).unwrap();
        assert!(
            s.graph().tasks[0].completed_at.is_none(),
            "completed_at should be None before completion"
        );
        s.start_task("t1").unwrap();
        s.complete_task("t1", None, None).unwrap();
        let task = s.graph().tasks.iter().find(|t| t.id == "t1").unwrap();
        assert!(
            task.completed_at.is_some(),
            "completed_at should be set after completion"
        );
    }

    #[test]
    fn summary_uses_completed_at_for_done_tasks() {
        use chrono::Duration;

        let dir = TempDir::new().unwrap();
        let mut s = Session::open(dir.path()).unwrap();
        let mut task = Task::new("t1", "A");
        // Simulate: started 10 seconds ago, completed 5 seconds ago.
        let now = Utc::now();
        task.status = Status::Done;
        task.started_at = Some(now - Duration::seconds(10));
        task.completed_at = Some(now - Duration::seconds(5));
        s.graph_mut().tasks.push(task);

        let summary = s.summary();
        let timing = &summary.tasks[0];
        // Duration should be ~5000ms (completed_at - started_at), NOT ~10000ms (now - started_at).
        assert!(
            timing.duration_ms < 7000,
            "duration_ms should use completed_at, got {}",
            timing.duration_ms
        );
        assert!(
            timing.duration_ms >= 4000,
            "duration_ms too small: {}",
            timing.duration_ms
        );
    }

    #[test]
    fn start_task_auto_flushes_to_disk() {
        let dir = TempDir::new().unwrap();
        let mut s = Session::open(dir.path()).unwrap();
        s.add_task(Task::new("t1", "A")).unwrap();
        s.start_task("t1").unwrap();
        // No explicit save() — reload from disk and verify state was flushed.
        let reloaded = graph::load(dir.path()).unwrap();
        let task = reloaded.tasks.iter().find(|t| t.id == "t1").unwrap();
        assert_eq!(task.status, Status::Running);
    }

    #[test]
    fn complete_task_auto_flushes_to_disk() {
        let dir = TempDir::new().unwrap();
        let mut s = Session::open(dir.path()).unwrap();
        s.add_task(Task::new("t1", "A")).unwrap();
        s.start_task("t1").unwrap();
        s.complete_task("t1", None, None).unwrap();
        let reloaded = graph::load(dir.path()).unwrap();
        let task = reloaded.tasks.iter().find(|t| t.id == "t1").unwrap();
        assert_eq!(task.status, Status::Done);
    }

    #[test]
    fn block_task_auto_flushes_to_disk() {
        let dir = TempDir::new().unwrap();
        let mut s = Session::open(dir.path()).unwrap();
        s.add_task(Task::new("t1", "A")).unwrap();
        s.start_task("t1").unwrap();
        s.block_task("t1", "stuck").unwrap();
        let reloaded = graph::load(dir.path()).unwrap();
        let task = reloaded.tasks.iter().find(|t| t.id == "t1").unwrap();
        assert_eq!(task.status, Status::Blocked);
    }

    #[test]
    fn unblock_task_auto_flushes_to_disk() {
        let dir = TempDir::new().unwrap();
        let mut s = Session::open(dir.path()).unwrap();
        s.add_task(Task::new("t1", "A")).unwrap();
        s.start_task("t1").unwrap();
        s.block_task("t1", "stuck").unwrap();
        s.unblock_task("t1").unwrap();
        let reloaded = graph::load(dir.path()).unwrap();
        let task = reloaded.tasks.iter().find(|t| t.id == "t1").unwrap();
        assert_eq!(task.status, Status::Pending);
    }

    #[test]
    fn session_summary_counts_correctly() {
        let dir = TempDir::new().unwrap();
        let mut s = Session::open(dir.path()).unwrap();
        s.add_task(Task::new("t1", "A")).unwrap();
        s.add_task(Task::new("t2", "B")).unwrap();
        s.start_task("t1").unwrap();
        let sum = s.summary();
        assert_eq!(sum.pending, 1);
        assert_eq!(sum.running, 1);
        assert_eq!(sum.done, 0);
    }

    // --- Config gating tests (fix #1: Session respects config) ---

    #[test]
    fn start_task_skips_rx_validation_when_disabled() {
        let dir = TempDir::new().unwrap();
        let mut cfg = crate::config::Config::default();
        cfg.integrations.rx = false;
        let mut s = Session::open_with_config(dir.path(), &cfg).unwrap();
        let mut task = Task::new("t1", "A");
        task.run = Some("rx:nonexistent-script".into());
        s.add_task(task).unwrap();
        // Should succeed — rx validation skipped when disabled
        assert!(s.start_task("t1").is_ok());
    }

    #[test]
    fn start_task_skips_cruxx_trace_when_disabled() {
        let dir = TempDir::new().unwrap();
        let mut cfg = crate::config::Config::default();
        cfg.integrations.cruxx = false;
        let mut s = Session::open_with_config(dir.path(), &cfg).unwrap();
        s.add_task(Task::new("t1", "A")).unwrap();
        s.start_task("t1").unwrap();
        // No session JSONL should exist when cruxx is disabled
        let sessions_dir = dir.path().join(".ctx").join("sessions");
        assert!(
            !sessions_dir.exists() || std::fs::read_dir(&sessions_dir).unwrap().count() == 0,
            "no trace files should be written when cruxx is disabled"
        );
    }

    // --- Session::clear_tasks test (fix #3) ---

    #[test]
    fn clear_tasks_auto_saves() {
        let dir = TempDir::new().unwrap();
        let mut s = Session::open(dir.path()).unwrap();
        s.add_task(Task::new("t1", "A")).unwrap();
        s.start_task("t1").unwrap();
        s.complete_task("t1", None, None).unwrap();
        let count = s.clear_tasks(true);
        assert_eq!(count, 1);
        let reloaded = graph::load(dir.path()).unwrap();
        assert!(reloaded.tasks.is_empty(), "clear_tasks should auto-save");
    }

    #[test]
    fn clear_tasks_done_only_keeps_pending() {
        let dir = TempDir::new().unwrap();
        let mut s = Session::open(dir.path()).unwrap();
        s.add_task(Task::new("t1", "A")).unwrap();
        s.add_task(Task::new("t2", "B")).unwrap();
        s.start_task("t1").unwrap();
        s.complete_task("t1", None, None).unwrap();
        let count = s.clear_tasks(true);
        assert_eq!(count, 1);
        assert_eq!(s.graph().tasks.len(), 1);
        assert_eq!(s.graph().tasks[0].id, "t2");
    }
}
