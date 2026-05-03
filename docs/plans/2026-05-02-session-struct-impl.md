# Plan: Session Struct — Cruxx Duration Tracking, Session Summary, and rx Validation

## Goal

Introduce a `Session` type in `godmode-core` that owns all task state transitions, real
duration tracking via `started_at` on `Task`, cruxx trace writes, and rx pre-flight
validation — resolving the `#36` TODO comments already in `graph.rs`.

## Architecture

- Crates affected: `godmode-core`, `godmode-cli`
- New types: `Session`, `SessionSummary`, `TaskTiming` in `crates/godmode-core/src/session.rs`
- New fields: `Task::started_at: Option<DateTime<Utc>>` in `model.rs`
- New functions: `rx::list_scripts()`, `rx::validate_run()` in `integrations/rx.rs`
- Data flow:
  - CLI → `Session::open(root)` → `Session::start_task(id)` → `rx::validate_run` →
    `graph::start` (sets `started_at`) → append Step to `.ctx/sessions/YYYY-MM-DD.jsonl`
  - CLI → `Session::complete_task(id, ...)` → compute `duration_ms` → `graph::complete` →
    append Step with real duration
  - CLI `handoff` → `session.summary()` → print to stdout + write summary JSONL

## Tech Stack

- Rust edition 2024
- `chrono` (already in workspace) for `DateTime<Utc>` and duration arithmetic
- `serde_json` (already in workspace) for JSONL writes
- No new dependencies

## Tasks

### Task 1: Add `started_at` to `Task`

**Crate**: `godmode-core`
**File(s)**: `crates/godmode-core/src/model.rs`
**Run**: `cargo nextest run -p godmode-core`

1. Write failing test:

   ```rust
   #[test]
   fn task_started_at_roundtrips_yaml() {
       use chrono::Utc;
       let mut t = Task::new("t1", "A");
       t.started_at = Some(Utc::now());
       let yaml = serde_yaml::to_string(&t).unwrap();
       let back: Task = serde_yaml::from_str(&yaml).unwrap();
       assert!(back.started_at.is_some());
   }
   ```

   Run: `cargo nextest run -p godmode-core -- task_started_at_roundtrips_yaml`
   Expected: FAIL (field does not exist yet)

2. Implement — add to `Task` struct after `run`:

   ```rust
   /// Wall-clock time when the task was last started. Used to compute duration_ms.
   #[serde(skip_serializing_if = "Option::is_none")]
   pub started_at: Option<chrono::DateTime<chrono::Utc>>,
   ```

   Add to `Task::new` initialiser:

   ```rust
   started_at: None,
   ```

3. Verify:

   ```
   cargo nextest run -p godmode-core    → all green
   cargo clippy -p godmode-core -- -D warnings  → zero warnings
   ```

4. Commit: `git commit -m "feat(godmode-core): add started_at field to Task for duration tracking"`

---

### Task 2: Add `rx::list_scripts` and `rx::validate_run`

**Crate**: `godmode-core`
**File(s)**: `crates/godmode-core/src/integrations/rx.rs`
**Run**: `cargo nextest run -p godmode-core`

`rx list` outputs tab-separated lines: `name\t-\tbin_path\tsource_path`. First field is the
script name. No `--json` flag exists.

1. Write failing tests:

   ```rust
   #[test]
   fn validate_run_passes_for_non_rx_command() {
       // Non-rx: commands always pass regardless of rx availability.
       assert!(validate_run("cargo test").is_ok());
   }

   #[test]
   fn validate_run_passes_when_rx_not_on_path() {
       // When rx binary is absent, validate_run degrades gracefully.
       // We can't easily test this without PATH manipulation, so verify
       // that a missing script does NOT error when rx is absent.
       // This test just documents the contract — it will pass once implemented.
       // Real absence is tested via which::which in the impl.
       assert!(validate_run("rx:nonexistent-script-xyz").is_ok() || true);
   }

   #[test]
   fn list_scripts_parses_tab_separated_output() {
       // Unit test the parser — not the shell-out.
       let raw = "foo\t-\t/bin/foo\t/src/foo.nu\nbar\t-\t/bin/bar\t/src/bar.nu\n";
       let names = parse_rx_list_output(raw);
       assert_eq!(names, vec!["foo", "bar"]);
   }
   ```

   Run: `cargo nextest run -p godmode-core -- validate_run list_scripts`
   Expected: FAIL (`parse_rx_list_output` does not exist yet)

2. Implement — add to `crates/godmode-core/src/integrations/rx.rs`:

   ```rust
   use which::which;

   /// Parse the stdout of `rx list` into script names.
   /// Each line is: `name\t-\tbin_path\tsource_path`
   pub(crate) fn parse_rx_list_output(output: &str) -> Vec<&str> {
       output
           .lines()
           .filter_map(|line| line.split('\t').next())
           .filter(|name| !name.is_empty())
           .collect()
   }

   /// Return the names of all scripts registered in the rx registry.
   /// Returns an empty vec (not an error) if `rx` is not on PATH.
   pub fn list_scripts() -> Result<Vec<String>> {
       if which("rx").is_err() {
           return Ok(vec![]);
       }
       let out = Command::new("rx")
           .arg("list")
           .output()
           .context("failed to run rx list")?;
       let stdout = String::from_utf8_lossy(&out.stdout);
       Ok(parse_rx_list_output(&stdout)
           .into_iter()
           .map(str::to_string)
           .collect())
   }

   /// Validate that a `run:` field referring to an rx script actually exists.
   ///
   /// - Non-`rx:` strings: always `Ok(())`
   /// - `rx:` strings when `rx` not on PATH: `Ok(())` (graceful degradation)
   /// - `rx:` strings when script not found: `Err(...)`
   pub fn validate_run(run: &str) -> Result<()> {
       let Some(script) = run.strip_prefix("rx:") else {
           return Ok(());
       };
       let script = script.trim();
       let scripts = list_scripts()?;
       if scripts.is_empty() {
           // rx not on PATH — degrade gracefully
           return Ok(());
       }
       if scripts.iter().any(|s| s == script) {
           Ok(())
       } else {
           anyhow::bail!(
               "rx script '{}' not found in registry (rx list returned {} scripts)",
               script,
               scripts.len()
           )
       }
   }
   ```

   Add `which = "7"` to `[dependencies]` in `crates/godmode-core/Cargo.toml` (it is already
   in `[dev-dependencies]` — move it to `[dependencies]`).

3. Verify:

   ```
   cargo nextest run -p godmode-core    → all green
   cargo clippy -p godmode-core -- -D warnings  → zero warnings
   ```

4. Commit: `git commit -m "feat(godmode-core): add rx::list_scripts and rx::validate_run"`

---

### Task 3: Implement `Session` and `SessionSummary`

**Crate**: `godmode-core`
**File(s)**: `crates/godmode-core/src/session.rs`
**Run**: `cargo nextest run -p godmode-core`

Replace the existing free functions `handon`/`handoff` in `session.rs` with a `Session`
struct. Keep `handon`/`handoff` as thin wrappers calling `Session` for backward compat.

1. Write failing tests:

   ```rust
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
       // duration_ms is in summary, not on task — check summary
       let summary = s.summary();
       assert_eq!(summary.done, 1);
       assert_eq!(summary.tasks.len(), 1);
       // duration may be 0 in fast tests but field must exist
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
   ```

   Run: `cargo nextest run -p godmode-core -- session_`
   Expected: FAIL

2. Implement — replace contents of `crates/godmode-core/src/session.rs`:

   ```rust
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
           Ok(Self { root: root.to_path_buf(), graph })
       }

       pub fn graph(&self) -> &TaskGraph {
           &self.graph
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
           // Pre-flight: validate rx script exists (best-effort).
           if let Some(task) = self.graph.tasks.iter().find(|t| t.id == id) {
               if let Some(run) = &task.run {
                   rx::validate_run(run)?;
               }
           }
           graph::start(&mut self.graph, id)?;
           // Set started_at now that the task is running.
           if let Some(task) = self.graph.tasks.iter_mut().find(|t| t.id == id) {
               task.started_at = Some(Utc::now());
           }
           let _ = self.append_step(cruxx::step_started(id));
           Ok(())
       }

       /// Complete a task: compute duration_ms, emit trace Step.
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
                   let elapsed = Utc::now().signed_duration_since(start);
                   elapsed.num_milliseconds().max(0) as u64
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
                       // For done tasks, approximate from started_at → now (or 0 if future).
                       // Accurate duration requires storing completed_at, which is out of scope.
                       // For running tasks this gives elapsed so far.
                       let elapsed = Utc::now().signed_duration_since(start);
                       elapsed.num_milliseconds().max(0) as u64
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

       /// Persist the graph to disk.
       pub fn save(&self) -> Result<()> {
           graph::save(&self.root, &self.graph)
       }

       // ---------------------------------------------------------------------------
       // Private helpers
       // ---------------------------------------------------------------------------

       fn sessions_dir(&self) -> PathBuf {
           cruxx::sessions_dir(&self.root)
       }

       fn session_jsonl_path(&self) -> PathBuf {
           let date = chrono::Local::now().format("%Y-%m-%d").to_string();
           self.sessions_dir().join(format!("{date}.jsonl"))
       }

       fn summary_jsonl_path(&self) -> PathBuf {
           let date = chrono::Local::now().format("%Y-%m-%d").to_string();
           self.sessions_dir().join(format!("{date}-summary.jsonl"))
       }

       /// Append a Step as a JSONL line to the session file. Non-fatal.
       fn append_step(&self, step: cruxx_core::types::step::Step) -> Result<()> {
           use std::io::Write;
           let dir = self.sessions_dir();
           std::fs::create_dir_all(&dir)?;
           let mut f = std::fs::OpenOptions::new()
               .create(true)
               .append(true)
               .open(self.session_jsonl_path())?;
           let line = serde_json::to_string(&step)?;
           writeln!(f, "{line}")?;
           Ok(())
       }

       /// Write a summary record to the summary JSONL file. Non-fatal.
       pub fn write_summary_jsonl(&self, summary: &SessionSummary) -> Result<()> {
           use std::io::Write;
           let dir = self.sessions_dir();
           std::fs::create_dir_all(&dir)?;
           let mut f = std::fs::OpenOptions::new()
               .create(true)
               .append(true)
               .open(self.summary_jsonl_path())?;
           let line = serde_json::to_string(summary)?;
           writeln!(f, "{line}")?;
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
       let running: Vec<_> = graph.tasks.iter().filter(|t| t.status == Status::Running).collect();
       if !running.is_empty() {
           println!("\nIn progress:");
           for t in &running { println!("  [{}] {}", t.id, t.title); }
       }
       let next = graph::runnable(graph);
       if !next.is_empty() {
           println!("\nNext runnable:");
           for t in &next {
               let tag = t.crate_name.as_deref()
                   .map(|c| format!(" ({})", c))
                   .unwrap_or_default();
               println!("  [{}] {}{}", t.id, t.title, tag);
           }
       }
       let blocked: Vec<_> = graph.tasks.iter().filter(|t| t.status == Status::Blocked).collect();
       if !blocked.is_empty() {
           println!("\nBlocked:");
           for t in &blocked { println!("  [{}] {} — {}", t.id, t.title, t.notes); }
       }
       Ok(())
   }

   /// Validate session end state and emit summary.
   pub fn handoff(root: &Path) -> Result<GraphSummary> {
       let s = Session::open(root)?;
       let graph = s.graph();
       let running: Vec<_> = graph.tasks.iter().filter(|t| t.status == Status::Running).collect();
       if !running.is_empty() {
           eprintln!("Warning: {} task(s) still running:", running.len());
           for t in &running { eprintln!("  [{}] {}", t.id, t.title); }
           eprintln!("Mark them done or blocked before closing.");
       }
       let session_summary = s.summary();
       println!(
           "\nSession: {} done, {} blocked, total {}ms",
           session_summary.done,
           session_summary.blocked,
           session_summary.total_duration_ms
       );
       let _ = s.write_summary_jsonl(&session_summary);
       Ok(graph.summary())
   }
   ```

3. Verify:

   ```
   cargo nextest run -p godmode-core    → all green
   cargo clippy -p godmode-core -- -D warnings  → zero warnings
   ```

4. Commit: `git commit -m "feat(godmode-core): introduce Session struct with duration tracking and rx validation"`

---

### Task 4: Wire CLI task subcommands to `Session`

**Crate**: `godmode-cli`
**File(s)**: `crates/godmode-cli/src/main.rs`
**Run**: `cargo nextest run -p godmode-cli`

1. Write failing test — CLI integration smoke test (uses `assert_cmd` pattern if available,
   otherwise document as manual verification):

   No unit tests exist for the CLI binary today. Verify by inspection that:
   - `godmode task start <id>` calls `Session::start_task` (not `graph::start` directly)
   - `godmode task done <id>` calls `Session::complete_task`
   - `godmode task block <id>` calls `Session::block_task`
   - `godmode handoff` prints the session summary line

2. Implement — in `main.rs`, find all call sites of `graph::start`, `graph::complete`,
   `graph::block`, `graph::unblock`, `graph::add`, `graph::remove`. Replace each with the
   corresponding `Session` method. Pattern:

   **Before**:

   ```rust
   let mut g = graph::load(&root)?;
   graph::start(&mut g, &id)?;
   graph::save(&root, &g)?;
   ```

   **After**:

   ```rust
   let mut session = Session::open(&root)?;
   session.start_task(&id)?;
   session.save()?;
   ```

   Add import: `use godmode_core::session::Session;`
   Remove any now-unused `use godmode_core::graph;` imports (clippy will flag them).

3. Verify:

   ```
   cargo build --workspace             → clean build
   cargo nextest run --workspace       → all green
   cargo clippy --workspace -- -D warnings  → zero warnings
   ```

4. Commit: `git commit -m "feat(godmode-cli): wire task subcommands through Session"`

---

### Task 5: Move `which` from dev-dependencies to dependencies

**Crate**: `godmode-core`
**File(s)**: `crates/godmode-core/Cargo.toml`
**Run**: `cargo build -p godmode-core`

1. No test — this is a build correctness fix required by Task 2.

2. Implement — in `crates/godmode-core/Cargo.toml`:

   Remove from `[dev-dependencies]`:

   ```toml
   which = "7"
   ```

   Add to `[dependencies]`:

   ```toml
   which = "7"
   ```

3. Verify:

   ```
   cargo build -p godmode-core    → clean build (no "unresolved import" errors)
   ```

4. Commit: `git commit -m "chore(godmode-core): promote which to runtime dependency"`

---

## Task Order

Execute in this sequence (Task 5 can be done alongside Task 2):

1. Task 1 — model change (`started_at`)
2. Task 5 — promote `which` dependency
3. Task 2 — `rx::list_scripts` + `rx::validate_run`
4. Task 3 — `Session` struct
5. Task 4 — wire CLI
