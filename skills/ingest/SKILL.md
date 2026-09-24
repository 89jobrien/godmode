---
name: "godmode:ingest"
description: >
  Use for "/gm:ingest", "/gm-ingest", "ingest plan", or "import plan", especially when loading
  multiple Markdown plans into one existing graph, avoiding task ID collisions, or verifying which
  tasks were added versus skipped.
requires: [task-management]
next: [task-driven-development]
---

# Ingest

Load one plan into the existing task graph without clearing history or colliding with tasks from
earlier plans.

## Resolve The Plan

1. Treat a supplied path as a literal file path. Never execute or interpolate it as shell syntax.
2. If no path is supplied, select the most recently modified Markdown file under
   `.ctx/godmode/plans/`.
3. Require an existing Markdown file. If a supplied path is missing, report it rather than adding an
   extension or selecting a different file silently.

## Validate Before Ingesting

Read the plan and verify:

- At least one `### Task N: <title>` heading exists.
- Plans produced by `godmode:writing-plans` have one `**Crate**:`, `**File(s)**:`, and `**Run**:`
  annotation per task. For manually authored plans, only task headings are mandatory.
- Task titles are non-empty.

Capture the current graph before mutation:

```bash
godmode task list --json
```

Do not clear the graph, remove tasks, or edit `.ctx/godmode/tasks.yaml` directly.

## Ingest

Run:

```bash
godmode --json plan ingest "/absolute/path/to/plan.md"
```

The CLI allocates IDs as follows:

- A first plan keeps its parsed `t1`, `t2`, ... IDs when they are unused.
- A later plan whose IDs collide with different tasks receives deterministic IDs prefixed by its
  sanitized file stem, such as `second-plan-t1`.
- Internal dependencies are rewritten into the same namespace.
- Prefix an explicit dependency with `graph:`, such as `graph:t1`, when it must reference an
  existing graph task instead of a task from the plan.
- Plan identity uses the resolved full path. Generated IDs start with the sanitized file stem and
  add a deterministic source hash when another plan already owns that namespace, so equal filenames
  in different directories remain distinct.
- Re-ingesting the same plan returns the same IDs and skips existing tasks.
- Shortening an already imported plan fails rather than orphaning its removed tasks. Reconcile those
  tasks explicitly before retrying.
- If a reserved ID contains different content, ingestion fails instead of silently dropping or
  overwriting work.

Require the JSON response to include `parsed`, `added`, `skipped`, and `ids`. Verify
`parsed == added + skipped`. If these fields are absent, the installed CLI predates collision-safe
ingestion; stop and update the binary before trusting the graph.

## Verify After Ingesting

Capture the graph and next runnable work:

```bash
godmode task list --json
godmode task next --json
```

Confirm every reported ID exists after ingestion and that each newly added dependency references
the intended task. An idempotent re-ingest with `added = 0` and `skipped = parsed` is successful.

## Report

Report:

- Exact plan path.
- Parsed, added, and skipped counts.
- Assigned task IDs.
- Next runnable task from this plan, plus any unrelated runnable work already in the graph.

On success, hand off to `godmode:task-driven-development` or `/gm:tdd`.
