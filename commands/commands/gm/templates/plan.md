## Rules

- Plan files go in `.ctx/tasks/`, not `.ctx/plans/`.
- Use `### Task N: <name>` headings with `**Crate**:`, `**File(s)**:`,
  `**Run**:` annotations.
- Every task must have: failing test, verify FAIL, implement, verify GREEN,
  commit.
- Each task should be 2-5 minutes of focused work.
- Run `godmode plan ingest <path>` to load tasks into the graph after writing.
- Task IDs are assigned sequentially per parse call — not from heading numbers.
