# godmode CLI Reference

## Session

```bash
godmode handon                          # triage: counts, running, next runnable, next doob todo
godmode handoff                         # session end: warns on running tasks
godmode status                          # fast mid-session check: counts + next runnable only
```

## Task Graph

```bash
godmode task list [--json]
godmode task add <id> "<title>" [--depends-on t1,t2] [--crate-name <crate>]
godmode task start <id>
godmode task done <id> [--commit <sha>] [--notes "<text>"]
godmode task block <id> "<reason>"
godmode task unblock <id>
godmode task remove <id>
godmode task clear --done               # prune completed tasks
godmode task clear --all                # reset graph entirely
godmode task next [--json]              # next runnable (exit 1 if none)
godmode task run <id> [--auto-done]     # run task's run: field; --auto-done marks done on exit 0
godmode task pull [--project <name>]    # import pending doob todos as tasks
godmode task push-done                  # mark completed tasks done in doob
```

## Plan Ingestion

```bash
godmode plan ingest <plan.md>           # idempotent — skips existing IDs
godmode agent <plan.md> [--max 5]       # ingest + dispatch in one step
```

## Dispatch

```bash
godmode dispatch [--max 5] [--json]     # independent chains for orca-strait
```

## Task File

`.ctx/GODMODE.tasks.yaml` — ephemeral, gitignored. Schema:

```yaml
tasks:
  - id: t1
    title: "Write failing test"
    status: done # pending | running | done | blocked
    crate_name: foo-core
    depends_on: []
    run: "cargo nextest run -p foo-core"
    commit: abc1234
    notes: ""
    completed: 2026-05-01
```

## Status Values

| Status    | Meaning                                         |
| --------- | ----------------------------------------------- |
| `pending` | Not started; runnable when deps are done        |
| `running` | Currently in progress                           |
| `done`    | Completed; unblocks dependents                  |
| `blocked` | Waiting on external factor; notes field has why |

## Dependency Rules

- A task is runnable when all `depends_on` entries are `done`
- Independent chains (no shared deps) can run in parallel via `godmode:parallel-agents`
- `godmode dispatch` identifies these chains automatically
