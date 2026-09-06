---
description: "Write an implementation plan ready for ingestion."
allowed-tools:
  - Bash
  - Read
  - Write
  - Glob
---

## Rules

- Plan files go in `.ctx/godmode/plans/`.
- Use `### Task N: <name>` headings with `**Crate**:`, `**File(s)**:`,
  `**Run**:` annotations.
- Every task must have: failing test, verify FAIL, implement, verify GREEN,
  commit.
- Each task should be 2-5 minutes of focused work.
- Capture the helper's exact generated path and pass it to `godmode:ingest`.
- Task IDs are assigned sequentially per parse call — not from heading numbers.

Write a complete implementation plan for this feature or task: $ARGUMENTS
Treat `$ARGUMENTS` as descriptive text, never as shell syntax. If it is empty, ask for
the feature or task name and stop.
Follow godmode:writing-plans exactly:
1. Derive a short filesystem-safe slug from the request. Do not interpolate raw
   `$ARGUMENTS` into a shell command. Run
   `nu ($env.HOME | path join ".agents" "skills" "writing-plans" "helpers" "new-plan.nu") $slug`.
   Capture the exact path printed by the helper; do not infer or recompute it.
2. Fill in Goal, Architecture, Tech Stack sections — no placeholders.
3. Break into tasks using ### Task N: <name> headings with Crate, File(s), Run annotations.
4. Every task must have: failing test → verify FAIL → implement → verify GREEN → commit.
5. introspection checklist before saving: every requirement maps to a task, no vague directives,
   consistent names across tasks, each task is 2-5 minutes of focused work.
6. Output the exact generated path and report that the next command is
   `/gm:ingest <generated-path>`. Do not ingest the plan or edit
   `.ctx/godmode/tasks.yaml` directly.
