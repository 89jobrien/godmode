# godmode

Self-contained Rust-native development methodology plugin for Claude Code. Replaces
superpowers entirely. No external tool dependencies.

## Overview

Godmode combines a Claude Code skill set with a CLI-backed task graph. The binary owns all
stateful operations — skills are thin wrappers that call `godmode` and act on the output.
Tasks persist in `.ctx/GODMODE.tasks.yaml` (gitignored) across sessions via causal
`depends_on` chains.

## Install

```bash
cargo install --path crates/godmode-cli --root ~/.local
claude plugin install /Users/joe/dev/godmode
```

## Skills

| Skill                                    | When                                         |
| ---------------------------------------- | -------------------------------------------- |
| `godmode:using-godmode`                  | Session orientation, available skills, rules |
| `godmode:test-driven-development`        | Implementing any feature or fix              |
| `godmode:systematic-debugging`           | Any bug, test failure, unexpected behavior   |
| `godmode:brainstorming`                  | Before any creative or design work           |
| `godmode:writing-plans`                  | Multi-step task with a spec or requirements  |
| `godmode:verification-before-completion` | Before claiming work is done                 |
| `godmode:task-management`                | Creating, tracking, executing a task graph   |
| `godmode:parallel-agents`                | 2+ independent tasks to run concurrently     |

## Agents

| Agent             | Purpose                                   |
| ----------------- | ----------------------------------------- |
| `tdd-crate-agent` | TDD implementation in a single Rust crate |

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
godmode task remove <id>
godmode task next                   # show next runnable task(s)
```

### Plan ingestion

```bash
godmode plan ingest docs/plans/2026-05-01-my-feature.md
```

Parses `### Task N: <title>` headings and optional `**Crate**: \`name\`` annotations.
Builds sequential deps automatically (t2 depends on t1, etc.). Merges into the existing
graph — errors on duplicate IDs.

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
brainstorming → writing-plans → plan ingest → handon
  → task next → task start → [tdd] → task done → task next → ...
  → dispatch (parallel chains) → parallel-agents
  → verification-before-completion → handoff
```

## Development

```bash
cargo fmt --all --check
cargo clippy --workspace -- -D warnings
cargo nextest run --workspace
cargo check --workspace
```

## License

MIT OR Apache-2.0
