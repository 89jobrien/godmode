# Plan: Critical path calculation (#20)

**Date:** 2026-05-02
**Issue:** #20
**Scope:** godmode-core + godmode-cli

## Goal

Add a critical path function to `dispatch` that identifies the longest dependency chain
among pending/running tasks. Exposed via `godmode dispatch --critical-path` and surfaced
in `godmode status`.

## Architecture

### Crates affected

- `crates/godmode-core/src/dispatch.rs` — new `critical_path` function
- `crates/godmode-cli/src/main.rs` — `--critical-path` flag on `Dispatch`, depth line in
  `Status`

### Algorithm

Longest path in a DAG via dynamic programming (topological sort + DP).

```
depth[t] = 1 + max(depth[dep] for dep in t.depends_on if dep is active)
```

Base case: tasks with no active deps have depth 1. Process in topological order (Kahn's
algorithm on the active subgraph). The task with the maximum depth is the tail of the
critical path; reconstruct by following the argmax parent pointer back to the root.

O(V + E) — same complexity as the existing `independent_chains`.

### New function

```rust
/// Return the longest dependency chain among pending/running tasks.
/// Ties broken by task insertion order (first root wins).
pub fn critical_path(graph: &TaskGraph) -> Vec<TaskRef>
```

Returns an empty `Vec` if no active tasks. Tasks are ordered root → tail (execution
order).

### CLI surface

**`godmode dispatch --critical-path`**

```
=== critical path (N tasks) ===
[t1] First task
[t3] Third task (depends on t1)
[t6] Sixth task (depends on t3)
```

With `--json`: `{"critical_path":[{"id":"t1","title":"..."},...], "depth":N}`

**`godmode status`** gains one new line in human output:

```
5 done  0 running  3 pending  0 blocked
  critical: 3 tasks deep
  next: [t1] First task
```

JSON output for `status` gains `"critical_depth": N`.

## Tech decisions

- DP on topological order — correct for DAGs, no recursion stack overflow risk.
- `critical_path` is a pure function on `&TaskGraph` — no mutation, easy to test.
- Reconstruction via parent pointers stored during DP — avoids a second traversal.
- `status` shows depth only (not full path) — avoids verbose output for the common case.
  Full path available via `dispatch --critical-path`.
- Ties broken by insertion order — deterministic, no randomness.

## Out of scope

- Weighted critical path (task duration estimates)
- Critical path across done tasks (history view)
- Slack calculation (how much each non-critical task can slip)

## Tasks

### Task 5: Implement `dispatch::critical_path`

Add `critical_path(graph: &TaskGraph) -> Vec<TaskRef>` to
`crates/godmode-core/src/dispatch.rs`. Use Kahn topological sort + DP on active
(pending/running) tasks. Reconstruct path via parent pointers.

**Crate**: `godmode-core`
**Run**: `cargo nextest run -p godmode-core`

### Task 6: Wire `dispatch --critical-path` CLI flag

Add `--critical-path` bool flag to `Cmd::Dispatch`. When set, call
`dispatch::critical_path` and emit the path list (human or JSON). Existing `--max`
dispatch output is unaffected when `--critical-path` is absent.

**Crate**: `godmode-cli`
**Run**: `cargo nextest run --workspace`

### Task 7: Surface critical depth in `godmode status`

In `Cmd::Status`, call `dispatch::critical_path` and include depth in both human and JSON
output. Human: `  critical: N tasks deep`. JSON: add `"critical_depth": N` field.

**Crate**: `godmode-cli`
**Run**: `cargo nextest run --workspace`
