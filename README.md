# godmode

Self-contained Rust-native development methodology plugin for Claude Code.

> Inspired by and built on the ideas in [superpowers](https://github.com/obra/superpowers) by [obra](https://github.com/obra) — the original agentic skills framework for Claude Code. Godmode replaces the superpowers runtime with a Rust-backed CLI and task graph, but the methodology, skill structure, and session ritual are directly descended from that work.

## Overview

Godmode combines a Claude Code skill set with a CLI-backed task graph. The binary owns all
stateful operations — skills are thin wrappers that call `godmode` and act on the output.
Tasks persist in `.ctx/GODMODE.tasks.yaml` (gitignored) across sessions via causal
`depends_on` chains.

## Install

```bash
cargo install --path crates/godmode-cli --root ~/.local
claude plugin install godmode@bazaar
```

## Skills

| Skill                                     | When                                          |
| ----------------------------------------- | --------------------------------------------- |
| `godmode:using-godmode`                   | Session orientation, available skills, rules  |
| `godmode:test-driven-development`         | Implementing any feature or fix               |
| `godmode:systematic-debugging`            | Any bug, test failure, unexpected behavior    |
| `godmode:brainstorm`                      | Before any creative or design work            |
| `godmode:writing-plans`                   | Multi-step task with a spec or requirements   |
| `godmode:verification-before-completion`  | Before claiming work is done                  |
| `godmode:task-management`                 | Creating, tracking, executing a task graph    |
| `godmode:parallel-agents`                 | 2+ independent tasks to run concurrently      |
| `godmode:code-review`                     | Quality pass before merge                     |
| `godmode:refactoring`                     | Restructure code without changing behaviour   |
| `godmode:receiving-review`                | Process incoming review feedback              |
| `godmode:cap`                             | Commit and push with validation               |
| `godmode:ci-fix`                          | Fix a failing CI pipeline                     |
| `godmode:tackle-issues`                   | Work GitHub issues in parallel worktrees      |
| `godmode:testing-philosophy`              | Choose the right test type for the situation  |
| `godmode:introspection`                   | Audit skills and plugin files for consistency |
| `godmode:observability-as-infrastructure` | Query and tail the session trace log          |

## Agents

| Agent                 | Purpose                                   |
| --------------------- | ----------------------------------------- |
| `godmode-crate-agent` | TDD implementation in a single Rust crate |
| `valerie`             | Task management and session orchestration |

## CLI Reference

### Session

```bash
godmode handon      # triage at session start — prints running, next, blocked
godmode handoff     # validate at session end — warns on tasks left running
```

### Task graph

```bash
godmode task list
godmode task add <id> <title> [--depends-on t1,t2] [--crate-name <crate>]
godmode task start <id>
godmode task done <id> [--commit <sha>] [--notes <text>]
godmode task block <id> <reason>
godmode task unblock <id>
godmode task remove <id>
godmode task clear --done           # prune completed tasks
godmode task clear --all            # reset graph entirely
godmode task next                   # show next runnable task(s)
godmode task run <id> [--auto-done] # run task's run: command; --auto-done marks done on exit 0
godmode task pull [--project <name>] # import pending doob todos as tasks
godmode task push-done               # mark completed tasks done in doob
```

### Status

```bash
godmode status                      # counts + next runnable, no external calls
```

### Plan ingestion

```bash
godmode plan ingest docs/plans/2026-05-01-my-feature.md
```

Parses `### Task N: <title>` headings, optional `**Crate**: \`name\``and`**Run**: \`cmd\`` annotations. Builds sequential deps automatically. Idempotent —
re-running a plan skips existing task IDs silently.

### Parallel dispatch

```bash
godmode dispatch [--max 5]
```

Emits a JSON array of independent task chains for parallel agent dispatch. Each chain
targets one crate. Cap defaults to 5 (API rate limit). Feed this output to
`godmode:parallel-agents`.

Example output:

```json
[
  { "crate_name": "godmode-core", "tasks": ["t1", "t2"] },
  { "crate_name": "godmode-cli", "tasks": ["t3"] }
]
```

## Task File

`.ctx/GODMODE.tasks.yaml` — ephemeral, gitignored. Schema:

```yaml
tasks:
  - id: t1
    title: "Write failing test for FooAdapter"
    status: done # pending | running | done | blocked
    depends_on: []
    crate_name: foo-core
    commit: abc1234
    notes: "test confirmed failing then green"
    completed: 2026-05-01

  - id: t2
    title: "Implement FooAdapter"
    status: pending
    depends_on: [t1]
    crate_name: foo-core
    notes: ""
```

## Workflow

```
brainstorm → writing-plans → plan ingest → handon
  → task next → task start → [tdd] → task done → task next → ...
  → dispatch (parallel chains) → parallel-agents
  → verification-before-completion → handoff
```

## Commands

Slash commands live in `commands/gm/` and map directly to skills. They can be invoked from
the Claude Code command palette (e.g. `/gm:cap`, `/gm:tdd`, `/gm:debug`).

## Helpers

Nushell helper scripts live in `skills/<skill>/helpers/`. Shared utilities are in
`skills/_lib/`:

| Module       | Purpose                                                     |
| ------------ | ----------------------------------------------------------- |
| `helpers.nu` | `repo-root`, `run-checked`, `cargo-gate`, `assert-not-main` |
| `trace.nu`   | Structured JSONL trace events (skill.start/complete/error)  |

Trace output lands in `.ctx/GODMODE.trace.jsonl`. Use `godmode:observability-as-infrastructure`
to query it.

## Development

```bash
cargo fmt --all --check
cargo clippy --workspace -- -D warnings
cargo nextest run --workspace
cargo check --workspace
```

## License

MIT OR Apache-2.0
