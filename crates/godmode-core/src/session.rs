use std::path::{Path, PathBuf};

use anyhow::Result;
use chrono::Utc;
use serde::Serialize;

use crate::graph;
use crate::integrations::{cruxx, rx};
use crate::model::{GraphSummary, Status, Task, TaskGraph};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

pub struct Session {
    root: PathBuf,
    graph: TaskGraph,
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
    /// Load (or create) a session rooted at `root`.
    pub fn open(root: &Path) -> Result<Self> {
        let graph = graph::load(root)?;
        Ok(Self {
            root: root.to_path_buf(),
            graph,
        })
    }

    pub fn graph(&self) -> &TaskGraph {
        &self.graph
    }

    pub fn graph_mut(&mut self) -> &mut TaskGraph {
        &mut self.graph
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
        if let Some(task) = self.graph.tasks.iter().find(|t| t.id == id)
            && let Some(run) = &task.run
        {
            rx::validate_run(run)?;
        }
        graph::start(&mut self.graph, id)?;
        if let Some(task) = self.graph.tasks.iter_mut().find(|t| t.id == id) {
            task.started_at = Some(Utc::now());
        }
        let _ = self.append_step(cruxx::step_started(id));
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

        let mut step = cruxx::step_completed(id, commit, notes);
        step.duration_ms = duration_ms;
        let _ = self.append_step(step);
        Ok(())
    }

    /// Block a task with a reason.
    pub fn block_task(&mut self, id: &str, reason: &str) -> Result<()> {
        graph::block(&mut self.graph, id, reason)?;
        let _ = self.append_step(cruxx::step_blocked(id, Some(reason)));
        Ok(())
    }

    /// Unblock a task.
    pub fn unblock_task(&mut self, id: &str) -> Result<()> {
        graph::unblock(&mut self.graph, id)
    }

    /// Unblock all blocked tasks. Returns count unblocked.
    pub fn unblock_all(&mut self) -> usize {
        graph::unblock_all(&mut self.graph)
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
            let duration_ms = task
                .started_at
                .map(|start| {
                    Utc::now()
                        .signed_duration_since(start)
                        .num_milliseconds()
                        .max(0) as u64
                })
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
            println!("  [{}] {}{}", t.id, t.title, tag);
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
}
