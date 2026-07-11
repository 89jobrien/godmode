# hooks/

Claude Code hook scripts and git hooks for the godmode plugin.

## Hook Registration

Hooks are registered in `hooks.json` and loaded by the Claude Code plugin system.
Valid `godmode hook run` names (verified against the binary):

```
stop-guard, auto-block, pre-commit, pre-commit-gate, quality-gate,
task-management, parallel-agents, moa, wave-integration, introspection,
agent-governance, brainstorm, code-review, ci-fix
```

Any other name passed to `godmode hook run` exits 1 with an "Unknown hook" error.

## hooks.json Events

| Event        | Matcher | Hooks                                                                          |
| ------------ | ------- | ------------------------------------------------------------------------------ |
| SessionStart | \*      | `session-start.rs`, `godmode hook run task-management`                         |
| PreToolUse   | Agent   | `pre-agent-task-context.nu`, `godmode hook run agent-governance`               |
| PreToolUse   | Bash    | `pre-bash-nag.nu`, `pre-commit-gate.nu`, `godmode hook run moa`                |
| PreToolUse   | Write   | `godmode hook run brainstorm`                                                  |
| PostToolUse  | Agent   | `check-blocked.sh`, `godmode hook run parallel-agents`                         |
| PostToolUse  | Bash    | `task-done-sync.nu`, `auto-block`, `ci-fix`, `code-review`, `wave-integration` |
| PostToolUse  | Write   | JSON/TOML/YAML/Nu validators, `post-write-plan-ingest.rs`                      |
| PostToolUse  | Edit    | JSON/TOML/YAML/Nu validators                                                   |
| Stop         | \*      | `godmode hook run stop-guard`, `memory-bank-update-remind.nu`                  |

## scripts/

| Script                         | Event        | Purpose                                                   |
| ------------------------------ | ------------ | --------------------------------------------------------- |
| `session-start.rs`             | SessionStart | Writes `session.start` trace event; generates session ID  |
| `pre-agent-task-context.nu`    | PreToolUse   | Injects current task context before Agent tool calls      |
| `pre-bash-nag.nu`              | PreToolUse   | Warns when pending tasks exist but none are running       |
| `pre-commit-gate.nu`           | PreToolUse   | Blocks commits on main; enforces branch discipline        |
| `check-blocked.sh`             | PostToolUse  | Checks for BLOCKED.md after Agent completes               |
| `post-json-validate.nu`        | PostToolUse  | Validates JSON syntax after Write/Edit                    |
| `post-toml-validate.nu`        | PostToolUse  | Validates TOML syntax after Write/Edit                    |
| `post-yaml-validate.nu`        | PostToolUse  | Validates YAML syntax after Write/Edit                    |
| `post-nu-check.nu`             | PostToolUse  | Syntax-checks `.nu` files after Write/Edit                |
| `post-write-plan-ingest.rs`    | PostToolUse  | Auto-ingests plan files written to `docs/plans/`          |
| `memory-bank-update-remind.nu` | Stop         | Reminds to update memory bank if files are stale          |
| `post-pipeline-step.nu`        | PostToolUse  | Advances pipeline state on `godmode pipeline next` output |
| `stop-guard.nu`                | (internal)   | Writes `session.end` trace event                          |

## Git Hooks

Installed per-repo via `nu hooks/install.nu` from the target repo root.

| Hook        | File             | Purpose                                          |
| ----------- | ---------------- | ------------------------------------------------ |
| pre-commit  | `pre-commit.nu`  | Blocks on running tasks; runs fmt/clippy/nextest |
| pre-push    | `pre-push.nu`    | Final validation before push                     |
| post-commit | `post-commit.nu` | Updates trace and doob state after commit        |

### Installing

```bash
# From any repo root:
nu /path/to/godmode/hooks/install.nu

# With $CLAUDE_PLUGIN_ROOT set:
nu "$CLAUDE_PLUGIN_ROOT/hooks/install.nu"
```

## lib/

Shared Nu modules used by hook scripts.

| Module                | Exports                                                      |
| --------------------- | ------------------------------------------------------------ |
| `godmode-hook-lib.nu` | `godmode-hook-context` — parses Claude tool event from stdin |
