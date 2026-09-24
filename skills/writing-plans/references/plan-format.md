# Plan Format Reference

## File Naming

`.ctx/godmode/plans/YYYY-MM-DD-<feature-name>.md`

Example: `.ctx/godmode/plans/2026-05-01-task-clear-command.md`

## Task Heading Format

```markdown
### Task N: <title>

**Crate**: `<crate-name>`
**Run**: `<shell command>`
```

- `### Task N:` — required prefix for `godmode plan ingest` to detect the task
- `**Crate**:` — optional; shown in `godmode task list` output
- `**Run**:` — optional; executed by `godmode task run <id> [--auto-done]`

## Run Annotation Behaviour

| Run value                  | How it executes                  |
| -------------------------- | -------------------------------- |
| `cargo nextest run -p foo` | Direct exec, split on whitespace |
| `rx:my-script`             | `rx run my-script`               |
| `echo hi > /tmp/out`       | `sh -c "echo hi > /tmp/out"`     |
| `cmd1 \| cmd2`             | `sh -c "cmd1 \| cmd2"`           |

Shell metacharacters (`>`, `<`, `\|`, `&`, `;`, `$`, `` ` ``, `(`, `)`) trigger `sh -c` automatically.

## Dependency Model

Tasks are assigned sequential deps automatically. The parser starts with `t1`; ingestion preserves
those IDs when unused or rewrites the whole plan into a deterministic file-stem namespace when they
collide with another plan.

To override the sequential dependency, add an explicit annotation:

```markdown
### Task 3: Add CI workflow

**Depends-on**: `graph:gh-42`
```

Use `t1,t2` for dependencies on tasks inside the same plan. Use `graph:<id>` for existing graph
tasks. Use an empty annotation, `**Depends-on**: ```, for an independent root task.

## Ingest Behaviour

- `godmode plan ingest <file>` — collision-safe and idempotent for the same plan path
- `godmode agent dispatch <file>` — uses the same collision-safe ingestion before dispatch
- Plan-ingest JSON reports `parsed`, `added`, `skipped`, and assigned `ids`; agent-dispatch JSON
  reports `parsed`, `ingested`, `skipped`, `ids`, and `chains`

## Quality Rules

| Rule             | Detail                                               |
| ---------------- | ---------------------------------------------------- |
| No placeholders  | Never write "TBD", "similar to Task N"               |
| Exact paths      | Every file path must be complete and correct         |
| Exact code       | Every code block must be copy-paste ready            |
| Consistent names | Types and methods must match across all tasks        |
| TDD every task   | Failing test → verify fail → implement → verify pass |
