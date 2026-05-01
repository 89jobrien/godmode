---
name: using-godmode
description: >
  Use at the start of every conversation to orient to available skills and workspace rules.
  Replaces superpowers entirely. Triggers on session start, "/godmode", or "what skills do
  you have".
---

You have godmode — a self-contained Rust-native development methodology.

## Instruction Priority

1. User's explicit instructions (CLAUDE.md, direct requests) — highest
2. Godmode skills — override defaults
3. Default system prompt — lowest

## The Rule

Invoke relevant skills BEFORE any response or action. 1% chance it applies = invoke it.

```
Message received → skill applies? → YES: invoke Skill tool first → NO: respond
```

## Red Flags (you are rationalizing — stop)

- "This is simple" → check anyway
- "I need context first" → skill check comes before exploration
- "The skill is overkill" → if it exists, use it
- "Just this one thing first" → check before acting

## Skill Priority

1. Process skills first: `godmode:brainstorming`, `godmode:systematic-debugging`
2. Implementation skills second: `godmode:test-driven-development`, `godmode:parallel-agents`

## Available Skills

| Skill                                    | When                                           |
| ---------------------------------------- | ---------------------------------------------- |
| `godmode:test-driven-development`        | Implementing any feature or fix                |
| `godmode:systematic-debugging`           | Any bug, test failure, unexpected behavior     |
| `godmode:brainstorming`                  | Before any creative or design work             |
| `godmode:writing-plans`                  | Multi-step task with a spec or requirements    |
| `godmode:verification-before-completion` | Before claiming work is done                   |
| `godmode:task-management`                | Creating, tracking, or executing a task graph  |
| `godmode:parallel-agents`                | 2+ independent tasks that can run concurrently |

## Always-Active Rules

- **Never `--no-verify`** on commits. Pre-commit hooks always run.
- **Commit signing**: SSH key via 1Password agent. Unlock 1Password and retry on failure.
- **Shell**: Nushell (`nu`) primary. No bash-isms (`&&`, `$()`, `export VAR=`) in `.nu` files.
- **Git in other repos**: `git -C <path>` not `cd <path> && git`.
- **Rust**: `cargo check` + `cargo clippy` after every change. `cargo nextest` over `cargo test`.
  Fix all clippy warnings before committing.
- **Secrets**: Never pass raw `op://` URIs. Use `op read` or `op run`.
- **Scope**: Touch only files within the explicitly requested scope.
- **Destructive ops**: Show diff, get explicit confirmation before dropping stashes or
  deleting files.

## Session Ritual

Start: check `.ctx/GODMODE.tasks.yaml` for pending tasks (see `godmode:task-management`).
End: update task statuses, note blockers.
