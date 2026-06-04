---
name: "godmode:task-management"
description: >
  Use when creating a task graph for a session, tracking progress across tasks, executing
  the next unblocked task, or managing causal dependencies between work items. Triggers on
  "create tasks", "what's next", "mark done", "task graph", or at session start when
  GODMODE.tasks.yaml exists.
requires: []
next: [task-driven-development, parallel-agents]
---

# Task Management

Godmode maintains a task graph at `.ctx/godmode/tasks.yaml` via the `godmode` CLI. Tasks
persist across sessions, encode causal dependencies, and drive sequential or parallel
execution. Never edit the YAML directly — always use the CLI.

## Session Start

```bash
godmode handon          # shows running, next runnable, blocked, next doob todo
godmode task next       # show only the next runnable task(s)
godmode task next --json  # machine-readable — exit 1 if empty
```

## Session End

```bash
godmode handoff         # warns on running tasks, calls hj handoff
```

## Task Operations

### Ingest from a plan doc

```bash
godmode plan ingest docs/plans/YYYY-MM-DD-<feature>.md
```

Parses `### Task N: <title>` headings, optional `**Crate**: \`name\``and`**Run**: \`cmd\`` annotations. Builds sequential deps automatically.

### Add a task manually

```bash
godmode task add <id> "<title>" [--depends-on t1,t2] [--crate-name <crate>]
```

### Start / complete / block / unblock

```bash
godmode task start <id>
godmode task done <id> [--commit <sha>] [--notes "<text>"]
godmode task block <id> "<reason>"
godmode task unblock <id>
godmode task unblock-all
```

### Remove a task

```bash
godmode task remove <id>
```

### List all tasks

```bash
godmode task list           # human table
godmode task list --json    # full JSON array — exit 1 if empty
```

## Rules

- A task is **runnable** when all `depends_on` entries have `status: done`.
- A task is **blocked** when a dependency is blocked — do not continue past it.
- Only one task per causal chain runs at a time.
- Independent chains (no shared deps) can run in parallel via `godmode:parallel-agents`.

## Parallel Dispatch

```bash
godmode dispatch [--max 5] --json
```

Emits independent chains shaped for orca-strait godmode-crate-agent. Each chain targets one
crate. Feed directly to `godmode:parallel-agents`.

## Run a Task's Shell Command

```bash
godmode task run <id>
```

Executes the `run:` field on the task. Prefix with `rx:` to invoke via rx registry.

## Example Workflow

```
godmode plan ingest docs/plans/2026-05-01-my-feature.md
godmode handon
godmode task start t1
# implement...
godmode task done t1 --commit abc1234
godmode task next          # → t2 is now runnable
godmode task start t2
# ...
godmode handoff
```

## Additional Resources

- **`references/godmode-cli.md`** — full CLI reference with all flags and task file schema
- **`helpers/session-workflow.md`** — common session patterns copy-paste ready
