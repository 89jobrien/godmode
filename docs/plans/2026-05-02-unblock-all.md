# Plan: `godmode task unblock-all` (#19)

**Date:** 2026-05-02
**Issue:** #19
**Scope:** godmode-core + godmode-cli

## Goal

Add a bulk-unblock operation that resets every `blocked` task to `pending` in one
command, eliminating the need to call `godmode task unblock <id>` once per task.

## Architecture

### Crates affected

- `crates/godmode-core/src/graph.rs` — new `unblock_all` function
- `crates/godmode-cli/src/main.rs` — new `TaskAction::UnblockAll` subcommand

### New function: `graph::unblock_all`

```rust
/// Reset all blocked tasks to pending, clearing their notes.
/// Returns the count of tasks unblocked.
pub fn unblock_all(graph: &mut TaskGraph) -> usize
```

- Iterates `graph.tasks`, flips `status = Blocked` → `status = Pending`, clears `notes`.
- Returns count for CLI output.
- No error cases — no-op (returns 0) if no blocked tasks.

### CLI subcommand

```
godmode task unblock-all
```

Added to `TaskAction` enum alongside `Unblock`. Calls `graph::unblock_all`, saves, prints:

- Human: `Unblocked N task(s).` (or `No blocked tasks.` when N=0)
- JSON: `{"ok":true,"unblocked":N}`

## Tech decisions

- Pure graph mutation, no trace event — unblocking is a graph bookkeeping op, not a
  lifecycle event worth tracing (unlike start/complete).
- Returns count (not ids) — callers who need ids can run `task list` before/after.
- No `--dry-run` — YAGNI; the operation is trivially reversible with `task block`.

## Out of scope

- Filtering by crate or status other than `blocked`
- Trace/cruxx events for unblock operations
- `unblock` (single-task) gaining any new behaviour

## Tasks

### Task 1: Add `graph::unblock_all`

Add `unblock_all(graph: &mut TaskGraph) -> usize` to `crates/godmode-core/src/graph.rs`.

**Crate**: `godmode-core`
**Run**: `cargo nextest run -p godmode-core`

### Task 2: Wire `task unblock-all` CLI subcommand

Add `TaskAction::UnblockAll` to `godmode-cli/src/main.rs`. Call `graph::unblock_all`,
save, emit human/JSON output.

**Crate**: `godmode-cli`
**Run**: `cargo nextest run --workspace`
