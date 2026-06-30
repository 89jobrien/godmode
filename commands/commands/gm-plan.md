---
name: plan
allowed_tools:
  - Bash
  - Read
  - Write
  - Glob
max_turns: 20
---

## Rules

- Plan files go in `.ctx/tasks/`, not `.ctx/plans/`.
- Use `### Task N: <name>` headings with `**Crate**:`, `**File(s)**:`,
  `**Run**:` annotations.
- Every task must have: failing test, verify FAIL, implement, verify GREEN,
  commit.
- Each task should be 2-5 minutes of focused work.
- Run `godmode plan ingest <path>` to load tasks into the graph after writing.
- Task IDs are assigned sequentially per parse call — not from heading numbers.

Write a complete implementation plan for a feature or task.
Follow godmode:writing-plans exactly:

1. Run skills/writing-plans/helpers/new-plan.nu "<feature-name>" to scaffold the file.
2. Fill in Goal, Architecture, Tech Stack sections — no placeholders.
3. Break into tasks using ### Task N: <name> headings with Crate, File(s), Run annotations.
4. Every task must have: failing test → verify FAIL → implement → verify GREEN → commit.
5. introspection checklist before saving: every requirement maps to a task, no vague directives,
   consistent names across tasks, each task is 2-5 minutes of focused work.
6. Run: godmode plan ingest <path> to load tasks into the graph.
   Output the plan file path when done.
