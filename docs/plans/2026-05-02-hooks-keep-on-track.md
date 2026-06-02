# Hooks That Keep On Track

**Date**: 2026-05-02
**Status**: done

## Goal

Add Claude Code hooks and git hooks that enforce godmode task discipline during a session and at
commit/push boundaries — so tasks can't silently drift, be orphaned, or be bypassed.

## What exists today

| Hook              | Location                         | Gap                                                 |
| ----------------- | -------------------------------- | --------------------------------------------------- |
| Stop — prompt     | `hooks/hooks.json`               | LLM-dependent; unreliable                           |
| PostToolUse/Bash  | `hooks/task-done-sync.nu`        | Only fires on `godmode task done`, not `git commit` |
| pre-commit        | `hooks/pre-commit.nu`            | Blocks on running tasks but not blocked tasks       |
| SessionStart      | `hooks/scripts/session-start.nu` | Works                                               |
| PostToolUse/Agent | `hooks/scripts/check-blocked.sh` | Works                                               |

## Changes

### 1. Stop hook — replace prompt with command (global)

**File**: `hooks/scripts/stop-guard.nu` (new)
**Registration**: `hooks/hooks.json` Stop entry — change `type: prompt` to `type: command`

Calls `godmode handoff --json`. If `running_count > 0`, exits 1 and prints blocking message.
Degrades gracefully if godmode not on PATH (exits 0).
Global registration means it works in all repos with a task file.

### 2. PostToolUse/Bash — git commit auto-advance

**File**: `hooks/task-done-sync.nu` (extend)

Add a second trigger: if the Bash command contains `git commit`, extract the commit SHA via
`git log -1 --format=%H`, find the first `running` task, call `godmode task done <id> --commit <sha>`.
Existing doob push-done logic runs after.

### 3. PreToolUse/Bash — no-running-task nag

**File**: `hooks/scripts/pre-bash-nag.nu` (new)
**Registration**: `hooks/hooks.json` PreToolUse/Bash entry (new)

Before any Bash command: if task file exists, `running == 0` and `pending > 0`, print a warning
to stderr. Always approves (never blocks). Degrades gracefully if godmode absent.

### 4. pre-commit — blocked task gate

**File**: `hooks/pre-commit.nu` (extend)

After the running-task check, add a blocked-task check: if any tasks have `status: blocked`,
fail the commit and list the blocked IDs + reasons. Forces the developer to resolve or explicitly
unblock before committing.

### 5. pre-push — orphaned running task gate

**File**: `hooks/pre-push.nu` (new)
**Installer**: `hooks/install.nu` (extend to also install pre-push)

Calls `godmode task list --json`. If any tasks are `running` with an empty `commit` field, fail
the push. A running task with a commit attached is fine (work is tracked); no commit means the
work is orphaned.

## Architecture

- All scripts degrade gracefully: `which godmode` check, fall through on absence.
- All scripts are Nushell (`.nu`) except `check-blocked.sh` which stays bash.
- Global hook (Stop) lives in `hooks/hooks.json` — already global via plugin install.
- Git hooks install to `.git/hooks/` via `hooks/install.nu`.

## Files touched

| File                            | Action                                                   |
| ------------------------------- | -------------------------------------------------------- |
| `hooks/hooks.json`              | Replace Stop prompt → command; add PreToolUse/Bash entry |
| `hooks/task-done-sync.nu`       | Extend with git-commit trigger                           |
| `hooks/scripts/stop-guard.nu`   | New                                                      |
| `hooks/scripts/pre-bash-nag.nu` | New                                                      |
| `hooks/pre-commit.nu`           | Extend blocked-task check                                |
| `hooks/pre-push.nu`             | New                                                      |
| `hooks/install.nu`              | Extend to install pre-push hook                          |

## Out of scope

- Pre-push blocking on all running tasks (only orphaned ones — running+commit is fine)
- Any changes to godmode-core
- Global git hooks config
