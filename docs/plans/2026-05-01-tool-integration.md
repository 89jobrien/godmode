# Plan: First-Class Tool Integration

**Date:** 2026-05-01
**Scope:** godmode-core + godmode-cli
**Goal:** Make doob, hj, rx, and orca-strait first-class citizens of the godmode CLI.
godmode becomes the compositor — routes to the right tool, never reimplements what they own.

## Design

### Integration boundaries

| Tool        | Owns                              | godmode role                            |
| ----------- | --------------------------------- | --------------------------------------- |
| doob        | Todo/backlog, kanban, sync        | Proxy for project-scoped todo ops       |
| hj          | HANDOFF.yaml lifecycle, reconcile | Thin wrapper for handon/handoff         |
| rx          | Script registry and runtime       | Resolve `run:` field on tasks           |
| orca-strait | Parallel TDD agent dispatch       | Emit dispatch JSON in orca-strait shape |

### JSON contract

Every command gains a top-level `--json` flag. Human-readable is the default.
Exit codes: 0 = success, 1 = empty/not-found, 2 = error.

### orca-strait dispatch format

tdd-crate-agent expects task titles, not just IDs. Output shape:

```json
[{ "crate_name": "godmode-core", "tasks": [{ "id": "t1", "title": "..." }] }]
```

---

## Tasks

### Task 1: Add --json flag and exit-code contract to godmode-cli

Add top-level `--json: bool` to `Cli`. Thread through every subcommand handler.
All handlers return structured output; `main` prints human or JSON accordingly.
Exit 1 when a list result is empty. Exit 2 on errors (unchanged via `?`).

**Crate**: `godmode-cli`

### Task 2: Implement hj integration in godmode-core

Add `src/integrations/hj.rs`. Expose:

- `handon(root: &Path) -> Result<String>` — calls `hj handon --project <name>`
- `handoff(root, build, tests, summary, commits) -> Result<String>` — calls `hj handoff` with args

Detect project name from nearest `Cargo.toml` `[package] name` (reuse `detect` module).
If `hj` is not on PATH, return a clear `Err` — never silently skip.

**Crate**: `godmode-core`

### Task 3: Implement doob integration in godmode-core

Add `src/integrations/doob.rs`. Expose:

- `todo_list(project: &str) -> Result<Value>` — calls `doob todo list -p <project> --json`
- `todo_next(project: &str) -> Result<Value>` — returns highest-priority pending item

Parse stdout as JSON. If `doob` is not on PATH, return a clear `Err`.

**Crate**: `godmode-core`

### Task 4: Wire hj + doob into handon/handoff commands

Replace `session::handon` / `session::handoff` with calls to the new integrations.

`godmode handon` (human): hj handon output, then next todo from doob.
`godmode handoff` (human): hj handoff output, then session summary counts.
`--json`: merge both as a single structured object.

**Crate**: `godmode-cli`

### Task 5: Fix dispatch output to match orca-strait format

Update `dispatch::independent_chains` return type to include task titles alongside IDs.

```rust
pub struct DispatchChain {
    pub crate_name: Option<String>,
    pub tasks: Vec<TaskRef>,
}
pub struct TaskRef {
    pub id: String,
    pub title: String,
}
```

Keep existing `--max` flag.

**Crate**: `godmode-core`, `godmode-cli`

### Task 6: Add rx integration and task run field

Add `run: Option<String>` to `model::Task`.
Update plan ingest to parse `**Run**: \`<command>\``annotations.
Add`src/integrations/rx.rs`: if `run`starts with`rx:`, calls `rx run <script>`;
otherwise shells out directly.
Add `godmode task run <id>`subcommand. Errors if task has no`run` field.

**Crate**: `godmode-core`, `godmode-cli`

### Task 7: Add --json to task list and task next

`task list --json`: full task graph as JSON array.
`task next --json`: next runnable task(s) as JSON array.
Both exit 1 when result is empty — LLMs use exit codes to branch.

**Crate**: `godmode-cli`
