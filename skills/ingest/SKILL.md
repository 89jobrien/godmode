---
name: "godmode:ingest"
description: >
  Use immediately after writing an implementation plan, or when the user says "ingest this
  plan", "load the plan", or invokes `/gm:ingest`. Converts one completed plan document into
  the godmode task graph before task-management or implementation begins.
requires: [writing-plans]
next: [task-management]
---

# Ingest

Convert one completed implementation plan into `.ctx/godmode/tasks.yaml` through the
`godmode` CLI. Never edit the task YAML directly.

## Resolve the Plan

1. If the user or upstream skill supplied a path, use that exact path.
2. Otherwise, use Glob to select the most recently modified Markdown file under
   `.ctx/godmode/plans/`.
3. If no plan exists, stop and report that `godmode:writing-plans` must run first.
4. Read the selected file and require at least one `### Task N: <title>` heading.

Do not guess a path from a feature name when `writing-plans` already returned an exact path.

## Ingest and Validate

Capture the graph before ingestion:

```bash
godmode task list --json
```

An empty graph may make this command exit 1; treat that as zero existing tasks. Then ingest
the selected plan, quoting its exact path:

```bash
godmode plan ingest "<exact-plan-path>"
```

Do not clear or replace existing tasks. Ingestion is additive and may skip task IDs already
present in the graph.

After ingestion, run:

```bash
godmode task list --json
godmode task next --json
```

The first command must return a readable task graph. `task next` may exit 1 when every task is
done or blocked; report that state rather than treating it as an ingestion failure.

## Report

Return:

- the exact plan path;
- the number of tasks added, computed from before/after graph counts;
- whether existing task IDs were skipped;
- the next runnable task or the reason none is runnable.

If ingestion fails, report the CLI error and stop. Do not repair the plan or task graph
silently; return to `godmode:writing-plans` for malformed plan content.

## Boundary with Task Management

This skill ends once the graph is validated and runnable work is reported. Use
`godmode:task-management` afterward for start, done, block, unblock, run, and dispatch
operations.
