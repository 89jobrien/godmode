# Agent UX Improvements

**Status**: done

Five changes to reduce friction for Claude Code when using godmode as its task
orchestration backbone. Ordered by time-saved-per-session.

---

## 1. Hook Preamble Library

### Goal

Eliminate the repeated 15-line boilerplate in every `.nu` hook (read stdin, find git
root, check task file, check godmode on PATH, parse JSON, type-guard). One import, one
function call, guaranteed correct types.

### Architecture

New file: `hooks/lib/godmode-hook-lib.nu`

Exports:

```nu
# Returns a record: {input: record, git_root: string, tasks: list, running: list, pending: list}
# Returns null if any precondition fails (not in repo, no task file, godmode missing).
export def godmode-hook-context [] -> record {
    ...
}
```

Each hook becomes:

```nu
use ../lib/godmode-hook-lib.nu [godmode-hook-context]
let ctx = (godmode-hook-context)
if $ctx == null { exit 0 }
# ... 3 lines of actual logic
```

### Files touched

- `hooks/lib/godmode-hook-lib.nu` (new, ~40 lines)
- All existing hooks in `hooks/scripts/` (refactor to use lib)
- `pre-agent-task-context.nu`, `post-write-plan-ingest.nu` (already written, refactor)

### Out of scope

- Changing hook registration format in `hooks.json`
- Adding new hook events

---

## 2. `godmode context --json`

### Goal

Single command that emits everything a hook or subagent needs to orient. Replaces
`godmode task list --json | parse | filter` in every consumer.

### Output shape

```json
{
  "git_root": "/Users/joe/dev/godmode",
  "project": "godmode",
  "running": [
    { "id": "t3", "title": "Implement X", "crate_name": "godmode-core" }
  ],
  "pending_count": 4,
  "blocked": [{ "id": "t5", "reason": "waiting on upstream" }],
  "recent_commits": ["951a05a fix(hooks): ...", "79329c6 fix(handoff): ..."],
  "critical_path_depth": 3
}
```

### Architecture

- **Crate**: `godmode-core` -- new `pub fn context(root: &Path) -> Result<SessionContext>`
- **CLI**: `godmode context [--json]` top-level subcommand (not nested under `task`)
- **Implementation**: composes `graph::load`, `graph::runnable`, `detect::project_name`,
  plus a `git log --oneline -5` subprocess call

### Files touched

- `crates/godmode-core/src/context.rs` (new module, ~60 lines)
- `crates/godmode-core/src/lib.rs` (add `pub mod context`)
- `crates/godmode-cli/src/main.rs` (add `Context` variant to `Cmd` enum)

### Out of scope

- Wave/worktree state in context (add later if needed)
- Session trace history

---

## 3. Exit Code Semantics

### Goal

Distinguish "no results" (exit 2) from "error" (exit 1) so hooks can branch without
parsing stderr.

### Convention

| Code | Meaning                                                                |
| ---- | ---------------------------------------------------------------------- |
| 0    | Success with results                                                   |
| 1    | Error (parse failure, IO error, invalid args)                          |
| 2    | Success but empty result set (no tasks match filter, no pending, etc.) |

### Architecture

- Replace `exit_empty()` in `main.rs` (currently always exits 1) with exit 2
- Affected commands: `task next`, `task list` (when filtered to empty), `dispatch`
  (no runnable chains)
- `godmode context` returns 0 even when empty (the context itself is the result)

### Files touched

- `crates/godmode-cli/src/main.rs` -- change `exit_empty` and all call sites

### Migration

- Existing hooks using `if $result.exit_code != 0` will now treat "empty" as success.
  This is the desired behavior -- hooks currently bail on empty which is correct.
- Document in CLAUDE.md under CLI subcommands section.

---

## 4. Auto-Block on Test Failure

### Goal

When `cargo nextest` (or `cargo test`) exits nonzero while a task is running,
automatically mark it blocked with the failure summary. No manual intervention needed.

### Architecture

New hook: `hooks/scripts/post-bash-auto-block.nu` (PostToolUse/Bash)

Logic:

1. Check if command contains `nextest run` or `cargo test`
2. Check if exit code in tool result is nonzero
3. Check if a task is currently running
4. Extract failure summary (first failing test name from stdout)
5. Run `godmode task block <id> "test failure: <summary>"`

### Files touched

- `hooks/scripts/post-bash-auto-block.nu` (new, ~35 lines)
- `hooks/hooks.json` (register under PostToolUse/Bash)

### Edge cases

- Multiple running tasks: block the one whose `crate_name` matches the `-p` flag
- No `-p` flag: block the first running task
- `--auto-done` on `task run`: skip auto-block (task run handles its own lifecycle)

### Out of scope

- Auto-unblock when tests pass (user should explicitly resume)
- Blocking on clippy failures (too noisy)

---

## 5. Simplify Pre-Agent Hook via `godmode context`

### Goal

Once `godmode context --json` exists (item 2), rewrite `pre-agent-task-context.nu` to
be 5 lines instead of 55.

### After

```nu
#!/usr/bin/env nu
use ../lib/godmode-hook-lib.nu [godmode-hook-context]
let ctx = (godmode-hook-context)
if $ctx == null { exit 0 }
if ($ctx.running | length) == 0 { exit 0 }
let lines = ($ctx.running | each { |t| $"- ($t.id): ($t.title) [($t.crate_name)]" })
print $"[godmode] Active task context for this agent:\n($lines | str join "\n")"
```

### Depends on

- Item 1 (hook preamble library)
- Item 2 (`godmode context`)

---

## Implementation Order

```
1. Hook preamble library (unblocks 4, 5)
2. `godmode context --json` (unblocks 5)
3. Exit code semantics (independent)
4. Auto-block on test failure (uses preamble from 1)
5. Rewrite pre-agent hook (uses 1 + 2)
```

### Task 1: Hook preamble library

**Crate**: hooks (nushell)
**Run**: `echo '{}' | nu hooks/lib/godmode-hook-lib.nu`

### Task 2: Add godmode context command

**Crate**: `godmode-core`
**Run**: `cargo nextest run -p godmode-core`

### Task 3: Exit code 2 for empty results

**Crate**: `godmode-cli`
**Run**: `cargo nextest run -p godmode-cli`

### Task 4: Auto-block on test failure hook

**Crate**: hooks (nushell)
**Run**: `echo '{"tool_input":{"command":"cargo nextest run"},"tool_result":{"exit_code":1,"stdout":"FAILED test_foo"}}' | nu hooks/scripts/post-bash-auto-block.nu`

### Task 5: Simplify pre-agent hook

**Crate**: hooks (nushell)
**Run**: `echo '{"tool_name":"Agent","tool_input":{"prompt":"test"}}' | nu hooks/scripts/pre-agent-task-context.nu`
