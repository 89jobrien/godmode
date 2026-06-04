# Session

Central orchestrator for task state transitions, duration tracking, and trace writes.

## Structure

- `root: PathBuf` — project root directory
- `graph: TaskGraph` — owned task graph
- `config: Config` — loaded from .godmode.toml

## Lifecycle

1. `Session::open(root)` — loads graph from disk, reads config
2. `start_task(id)` — validates via rx, sets started_at, writes cruxx Step
3. `complete_task(id, commit, notes)` — records completed_at, duration, cruxx Step
4. `handoff()` — writes SessionSummary to JSONL, returns summary

## Key constraint

All task transitions MUST go through Session (not raw graph functions)
for duration tracking and trace writes to work correctly.

## Defined in

`crates/godmode-core/src/session.rs`

## Related

- [[Task]] / [[TaskGraph]] — data model
- [[cruxx]] — trace Step writes
- [[rx]] — run command validation
