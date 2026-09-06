## Rules

- Plan files go in `.ctx/godmode/plans/`.
- Use `### Task N: <name>` headings with `**Crate**:`, `**File(s)**:`,
  `**Run**:` annotations.
- Every task must have: failing test, verify FAIL, implement, verify GREEN,
  commit.
- Each task should be 2-5 minutes of focused work.
- Capture the helper's exact generated path and pass it to `godmode:ingest`.
- Task IDs are assigned sequentially per parse call — not from heading numbers.
