# Plan: Cycle detection in `graph::add` (#18)

**Date:** 2026-05-02
**Issue:** #18
**Scope:** godmode-core

## Goal

`graph::add` currently accepts any `depends_on` list without validating the dependency
graph. A cycle (direct or transitive) causes `graph::runnable` to return empty silently —
no task ever becomes runnable, with no diagnostic. This plan adds cycle detection at
insertion time so the graph is guaranteed to be a DAG.

## Architecture

### Crates affected

- `crates/godmode-core/src/graph.rs` — extend `graph::add`

### Algorithm

DFS reachability check at insertion time. When adding task `T` with `depends_on = [d1, d2,
...]`:

1. Build a transient adjacency map from the existing graph (id → depends_on).
2. From each `di`, do a depth-first walk following `depends_on` edges.
3. If the walk reaches `T.id`, a cycle exists — collect the path and return `Err`.

This is O(V + E) per insertion — acceptable for the small graphs godmode targets (<100
tasks).

### Error message

```
cycle detected: t3 → t1 → t3
```

Path is built during the DFS backtrack and included in the `anyhow::bail!` message.

### Signature — no change

`graph::add` already returns `Result<()>`. The new error variant is a new `Err` case;
callers using `.ok()` to skip duplicates will also silently skip cycles — callers that
care (CLI, plan ingest) will surface the error automatically.

### Plan ingest

`plan::parse` → `graph::add` in a loop. Currently errors on duplicate IDs are swallowed
with `.ok()`. Cycle errors should NOT be swallowed — a plan with a cycle is malformed.
Update the ingest call site in `main.rs` to propagate cycle errors.

## Tech decisions

- DFS at insertion (not at load/save) — catches errors at the earliest possible point.
- Path collection during DFS — more useful than just "cycle exists".
- No structural change to `Task` or `TaskGraph` — purely algorithmic.
- `plan ingest` propagates cycle errors, `graph::add` callers using `.ok()` swallow them
  (intentional — duplicate-skip pattern is preserved).

## Out of scope

- Cycle detection at load time (graphs on disk are assumed valid)
- Detecting cycles in `depends_on` referencing non-existent tasks (separate concern)
- UI for visualising cycles

## Tasks

### Task 3: Implement cycle detection in `graph::add`

Add a private `would_create_cycle(graph: &TaskGraph, new_id: &str, deps: &[String]) ->
Option<String>` helper that returns `Some(path)` if a cycle would be created, `None`
otherwise. Call it from `graph::add` before pushing.

**Crate**: `godmode-core`
**Run**: `cargo nextest run -p godmode-core`

### Task 4: Propagate cycle errors in plan ingest

In `main.rs` `PlanAction::Ingest` and `Cmd::Agent`, replace `.ok()` on `graph::add` with
logic that swallows `"already exists"` errors but propagates cycle errors.

**Crate**: `godmode-cli`
**Run**: `cargo nextest run --workspace`
