# Plan: Plugin Layer, Integration Tests, and Agent Command

**Date:** 2026-05-01
**Scope:** godmode plugin + godmode-core + godmode-cli

## A — Plugin layer

The skills exist but `plugin.json` has no `skills` block, so `claude plugin install` installs
nothing callable. Skills also reference manual YAML editing instead of the CLI. Fix both.

## C — Integration tests

`integrations/hj.rs`, `doob.rs`, `rx.rs` shell out with no test coverage. Use a fake-binary
pattern: write minimal shell stubs to a temp dir, prepend to PATH, assert on stdout/exit code.

## B — `godmode agent` command

Takes a plan doc, ingests it, runs dispatch, emits a structured prompt that hands off to
orca-strait godmode-crate-agent. The compositor command that closes the loop.

---

## Tasks

### Task 1: Register skills in plugin.json

Add a `skills` array to `.claude-plugin/plugin.json` listing all 8 skill directories.
Format matches orca-strait's convention: `{ "name": "<skill-name>", "path": "skills/<name>" }`.
Verify `claude plugin install /Users/joe/dev/godmode` picks them up after the change.

**Crate**: `godmode-cli`

### Task 2: Update task-management skill to use CLI

Rewrite `skills/task-management/SKILL.md` — replace all manual YAML editing instructions
with `godmode` CLI invocations. Every operation (add, start, done, block, unblock, next,
list) maps to a CLI call. Keep the conceptual explanation, replace the mechanics.

**Crate**: `godmode-cli`

### Task 3: Update using-godmode skill session ritual

Update `skills/using-godmode/SKILL.md` session ritual section:

- Start: `godmode handon` (not "check .ctx/GODMODE.tasks.yaml manually")
- End: `godmode handoff`
  Add a `## CLI Reference` section with the most-used commands as a quick cheat sheet.

**Crate**: `godmode-cli`

### Task 4: Add fake-binary test harness to godmode-core

Add `crates/godmode-core/tests/fake_bin.rs` (integration test file).
Define a helper `FakeBin` that:

- Writes a minimal shell script to a `TempDir` that echoes fixed JSON to stdout and exits 0
- Returns a modified `PATH` string with the temp dir prepended
  Use this harness in subsequent test tasks — it is the shared fixture, not a test itself.

**Crate**: `godmode-core`

### Task 5: Integration tests for doob integration

Add tests in `crates/godmode-core/tests/doob_integration.rs` using `FakeBin`:

- `todo_list_parses_json`: fake `doob` echoes valid JSON; assert `todo_list` returns it
- `todo_next_returns_first_pending`: fake `doob` returns two todos (one completed, one
  pending); assert `todo_next` returns the pending one
- `todo_list_errors_on_nonzero`: fake `doob` exits 1; assert `todo_list` returns `Err`

**Crate**: `godmode-core`

### Task 6: Integration tests for hj integration

Add tests in `crates/godmode-core/tests/hj_integration.rs` using `FakeBin`:

- `handon_returns_stdout`: fake `hj` echoes a known string; assert `hj::handon` returns it
- `handoff_passes_args`: fake `hj` echoes its own argv as JSON; assert `--build`, `--tests`,
  `--log-summary`, and `--commit` args are present
- `handon_errors_when_hj_missing`: use a PATH with no `hj`; assert `Err` with helpful message

**Crate**: `godmode-core`

### Task 7: Integration tests for rx integration

Add tests in `crates/godmode-core/tests/rx_integration.rs` using `FakeBin`:

- `direct_cmd_runs`: fake `echo` binary; assert exit 0
- `rx_prefix_delegates_to_rx`: fake `rx` binary that echoes argv; assert `rx run <script>`
  is called when run string is `rx:my-script`
- `nonzero_exit_propagated`: fake binary exits 2; assert `run_cmd` returns status with
  `success() == false`

**Crate**: `godmode-core`

### Task 8: Add `godmode agent <plan>` command

Add `Cmd::Agent { path: String }` to the CLI.

Behaviour:

1. Read the plan markdown from `path`
2. Call `plan::parse` — error if 0 tasks
3. Load existing graph; ingest tasks (skip duplicates via `graph::add` returning `Err`)
4. Call `dispatch::independent_chains(&g, 5)`
5. Emit a structured handoff prompt to stdout:

```
=== godmode agent dispatch ===
Plan: <path>
Chains: <N>

<JSON chains array>

Paste the above into orca-strait or feed to godmode-crate-agent directly.
```

With `--json`: emit `{ "plan": "<path>", "chains": [...] }` only.

**Crate**: `godmode-cli`
