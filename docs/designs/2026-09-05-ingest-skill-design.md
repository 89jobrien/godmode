# Design: Ingest Skill and Command

## Goal

Add a focused `godmode:ingest` workflow step that converts a completed implementation plan into the godmode task graph before task management begins.

## Approved Approach

Extract plan ingestion from `task-management`, expose it as both a skill and `/gm:ingest` command, and delegate to it from `writing-plans` and planning pipelines.

## Ownership

- **Skill owner**: `skills/ingest/SKILL.md` owns plan selection, ingestion, validation, and reporting.
- **Command source**: `command-support/gm/ingest.yaml` provides direct `/gm:ingest` invocation; `commands/gm-ingest.md` remains generated output.
- **Existing CLI**: `godmode plan ingest <path>` remains the execution mechanism. No Rust API or CLI changes are required.

## Command Contract

```text
/gm:ingest [plan-path]
```

- With a path, ingest that exact file.
- Without a path, select the newest Markdown plan in `.ctx/godmode/plans/`.
- Reject an absent plan or a document without `### Task N:` headings before invoking the CLI.
- Run `godmode plan ingest <path>`, then inspect `godmode task list --json` and `godmode task next --json`.
- Report the selected plan, imported task count, and next runnable task or tasks.

## Workflow

1. `writing-plans` writes and reports the plan path.
2. `ingest` resolves that explicit path or discovers the newest plan.
3. The existing CLI parser creates sequential task dependencies in `.ctx/godmode/tasks.yaml`.
4. `ingest` validates the resulting graph and reports runnable work.
5. `task-management` owns subsequent start, done, block, unblock, run, and dispatch operations.

## Integration Points

- Change `writing-plans` metadata from `next: [task-management]` to `next: [ingest]` and delegate its after-writing action.
- Remove plan-ingestion ownership from `task-management`; retain graph lifecycle operations there.
- Insert `ingest` between `writing-plans` and `task-management` in applicable pipelines.
- Register `ingest` in skill discovery indexes and command documentation.

## Out of Scope

- Changing plan parsing or task ID assignment.
- Clearing or replacing an existing task graph automatically.
- Changing the `godmode plan ingest` CLI.
- Removing the existing post-write auto-ingest hook.

## Risk

- [x] Breaking API changes: no.
- [x] New external dependency: no.
- [x] Serialization changes: no.
- [x] Generated artifacts: regenerate command and skill indexes rather than editing them manually.
- [x] Duplicate ingestion: existing CLI idempotency remains authoritative; the skill must report skipped or unchanged tasks accurately.
