# Design: Session Struct — Cruxx Duration Tracking, Session Summary, and rx Validation

**Date**: 2026-05-02
**Author**: Joseph O'Brien
**Status**: done

## Goal

Introduce a `Session` type that owns all task state transitions, cruxx trace writes, and rx
pre-flight validation. Currently `graph::start_traced`/`complete_traced` build `Step` values
and immediately discard them; `duration_ms` is always 0; `rx:` script existence is never
checked. This design wires those three capabilities together in one cohesive layer.

## Background

`graph.rs` already has `// Trace step is recorded via Session::record in the session_trace
layer (#36)` comments — the `Session` struct was always the intended home for trace writes.
Duration tracking requires `started_at` on `Task` (so it survives save/load). rx validation
belongs in `Session::start_task` so it fires before state is mutated.

## Architecture

### Crates Affected

| Crate          | Change                                                            |
| -------------- | ----------------------------------------------------------------- |
| `godmode-core` | Add `started_at` to `Task`; new `Session` type in `session.rs`;   |
|                | new `rx::list_scripts` + `rx::validate_run`; new `SessionSummary` |
| `godmode-cli`  | CLI task subcommands call `Session` methods instead of `graph::*` |

### New Types

```rust
// model.rs — add to Task
pub started_at: Option<DateTime<Utc>>,   // set on start, used to compute duration_ms

// session.rs
pub struct Session {
    root: PathBuf,
    graph: TaskGraph,
}

pub struct SessionSummary {
    pub done: usize,
    pub running: usize,
    pub pending: usize,
    pub blocked: usize,
    pub total_duration_ms: u64,
    pub tasks: Vec<TaskTiming>,
}

pub struct TaskTiming {
    pub id: String,
    pub title: String,
    pub duration_ms: u64,   // 0 if not completed this session
}

impl Session {
    pub fn open(root: &Path) -> Result<Self>;
    pub fn start_task(&mut self, id: &str) -> Result<()>;
    pub fn complete_task(
        &mut self,
        id: &str,
        commit: Option<&str>,
        notes: Option<&str>,
    ) -> Result<()>;
    pub fn block_task(&mut self, id: &str, reason: &str) -> Result<()>;
    pub fn unblock_task(&mut self, id: &str) -> Result<()>;
    pub fn add_task(&mut self, task: Task) -> Result<()>;
    pub fn remove_task(&mut self, id: &str) -> Result<()>;
    pub fn graph(&self) -> &TaskGraph;
    pub fn summary(&self) -> SessionSummary;
    pub fn save(&self) -> Result<()>;           // explicit — caller decides when to persist
}

// integrations/rx.rs — new additions
pub fn list_scripts() -> Result<Vec<String>>;   // calls `rx list`, parses names
pub fn validate_run(run: &str) -> Result<()>;   // no-op if not rx:, no-op if rx not on PATH
```

### Data Flow

**Task start**:

1. `Session::start_task(id)` calls `rx::validate_run(task.run)` — errors if `rx:script` not
   found (best-effort; skipped if `rx` binary absent)
2. `graph::start(&mut self.graph, id)` mutates status to Running, sets `task.started_at`
3. Builds `cruxx::step_started(id)`, appends to `.ctx/sessions/<date>.jsonl` (non-fatal)
4. Caller calls `session.save()` to persist

**Task complete**:

1. `graph::complete(...)` mutates status to Done, sets `task.completed`
2. `duration_ms` = `Utc::now() - task.started_at` (0 if `started_at` absent)
3. Builds `cruxx::step_completed(id, commit, notes)` with real `duration_ms`, appends to
   session JSONL (non-fatal)
4. Caller calls `session.save()`

**Session handoff**:

1. `session.summary()` aggregates counts + durations from graph
2. Prints human-readable summary to stdout
3. Writes `SessionSummary` as a single JSONL record to `.ctx/sessions/<date>-summary.jsonl`
   (non-fatal)

### Session file location

`.ctx/sessions/YYYY-MM-DD.jsonl` — one Step per line, appended on each transition.
`.ctx/sessions/YYYY-MM-DD-summary.jsonl` — one record written at `handoff` time.
Both paths are gitignored (already covered by `.ctx/` gitignore entry).

## Approaches Considered

### Option A: Incremental (duration → summary → rx validation in sequence)

Three independent PRs. Each piece shippable alone. Trade-off: `started_at` is needed by all
three; splitting means the model change lands first in isolation.

### Option B: Parallel workstreams (cruxx + rx independently)

Duration/summary together, rx separately. Cohesive units but requires coordinating two
branches and merging.

### Option C: Session-centric (chosen)

One `Session` type owns all three capabilities. Single model change (`started_at`), single
new public API surface, one commit. Resolves the `#36` TODO comments already in graph.rs.

## Tech Decisions

| Decision                                        | Rationale                                                      |
| ----------------------------------------------- | -------------------------------------------------------------- |
| Explicit `session.save()` not auto-save         | Matches existing `graph::save` pattern; easier to test;        |
|                                                 | avoids partial-write bugs on multi-step mutation sequences     |
| `SessionSummary` printed to stdout + JSONL      | Human output for interactive use; JSONL for future querying    |
| `rx::validate_run` degrades gracefully          | `rx` may not be installed; validation is best-effort preflight |
| `graph::start`/`complete` stay as pure mutators | Keeps graph layer testable without Session overhead;           |
|                                                 | Session delegates to them internally                           |
| `started_at` on `Task`, not in Session map      | Survives save/load across process restarts (e.g. CLI calls)    |

## Out of Scope

- `rx list --json` output format stabilisation (we parse whatever `rx list` emits today)
- Querying historical session JSONL files (`godmode session history` etc.)
- Confidence scoring on `Step` (remains 1.0)
- Changing the `--json` output format of any existing subcommand
- Session locking / concurrent writer safety

## Open Questions

- [ ] Does `rx list` emit JSON today, or does it need `--json`? Check `rx` CLI before
      implementing `list_scripts`.
