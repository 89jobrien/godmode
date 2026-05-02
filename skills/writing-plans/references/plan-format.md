# Plan Format Reference

## File Naming

`docs/plans/YYYY-MM-DD-<feature-name>.md`

Example: `docs/plans/2026-05-01-task-clear-command.md`

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

Tasks are assigned sequential deps automatically: t2 depends on t1, t3 on t2, etc.

To make a task independent (no deps), include "independent" in the title:

```markdown
### Task 3: Independent — add CI workflow
```

## Ingest Behaviour

- `godmode plan ingest <file>` — idempotent; skips existing task IDs silently
- `godmode agent <file>` — ingest + dispatch in one step (also idempotent)

## Quality Rules

| Rule             | Detail                                               |
| ---------------- | ---------------------------------------------------- |
| No placeholders  | Never write "TBD", "similar to Task N"               |
| Exact paths      | Every file path must be complete and correct         |
| Exact code       | Every code block must be copy-paste ready            |
| Consistent names | Types and methods must match across all tasks        |
| TDD every task   | Failing test → verify fail → implement → verify pass |
