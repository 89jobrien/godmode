---
name: task-management
description: >
  Use when creating a task graph for a session, tracking progress across tasks, executing
  the next unblocked task, or managing causal dependencies between work items. Triggers on
  "create tasks", "what's next", "mark done", "task graph", or at session start when
  GODMODE.tasks.yaml exists.
---

# Task Management

Godmode maintains a self-contained task graph at `.ctx/GODMODE.tasks.yaml`. No external
tools required. Tasks persist across sessions, encode causal dependencies, and drive
sequential execution.

## Task File Location

`.ctx/GODMODE.tasks.yaml` — ephemeral, gitignored. Create `.ctx/` if it doesn't exist.

Add to `.gitignore`:

```
.ctx/
```

## Schema

```yaml
# .ctx/GODMODE.tasks.yaml
tasks:
  - id: t1
    title: "Write failing test for FooAdapter"
    status: done # pending | running | done | blocked
    depends_on: []
    notes: "Completed, all green"

  - id: t2
    title: "Implement FooAdapter"
    status: running
    depends_on: [t1]
    notes: ""

  - id: t3
    title: "Wire FooAdapter into service layer"
    status: pending
    depends_on: [t2]
    notes: ""

  - id: t4
    title: "Integration tests for Foo"
    status: pending
    depends_on: [t3]
    notes: ""
```

## Rules

- A task is **runnable** when all entries in `depends_on` have `status: done`.
- A task is **blocked** when a dependency has `status: blocked`.
- Only one task per causal chain runs at a time (`status: running`).
- Independent chains (no shared dependencies) can run in parallel via `godmode:parallel-agents`.

## Operations

### Create task graph

Read the plan from `docs/plans/YYYY-MM-DD-<feature>.md`, extract tasks, write the YAML.
Assign IDs sequentially (`t1`, `t2`, ...). Set all statuses to `pending`.

### Find next runnable task

```
for each task where status == pending:
  if all depends_on tasks have status == done:
    this task is runnable
```

### Start a task

Set `status: running`. Update the file. Execute the task.

### Complete a task

Set `status: done`. Add notes summarizing what was done and the commit SHA.
Find the next runnable task and continue.

### Block a task

Set `status: blocked`. Add notes explaining why (3 attempts failed, dependency issue, etc.).
Surface the blocker to the user — do not continue past a blocked task in the same chain.

### Session start

If `.ctx/GODMODE.tasks.yaml` exists, read it and print a summary:

```
Tasks: 2 done, 1 running, 3 pending, 0 blocked
Next runnable: t4 — "Integration tests for Foo"
```

### Session end

Ensure all `running` tasks are updated to `done` or `blocked` before closing.

## Independent Chains

When the task graph has independent chains with no shared dependencies, they can run in
parallel. See `godmode:parallel-agents` for the dispatch protocol.

Example — parallel chains:

```yaml
tasks:
  - id: a1
    title: "Implement AuthAdapter"
    depends_on: []
  - id: a2
    title: "Test AuthAdapter"
    depends_on: [a1]
  - id: b1
    title: "Implement CacheAdapter"
    depends_on: [] # independent of a-chain
  - id: b2
    title: "Test CacheAdapter"
    depends_on: [b1]
```

`a1` and `b1` are runnable simultaneously. Dispatch as parallel agents.
