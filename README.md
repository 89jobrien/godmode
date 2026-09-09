# godmode

Self-contained Rust-native development methodology for Claude Code, Codex, and OpenCode.

> Inspired by and built on the ideas in [superpowers](https://github.com/obra/superpowers) by [obra](https://github.com/obra) — the original agentic skills framework for Claude Code. Godmode replaces the superpowers runtime with a Rust-backed CLI and task graph, but the methodology, skill structure, and session ritual are directly descended from that work.

## Overview

Godmode combines agent skill sets with a CLI-backed task graph. The binary owns all
stateful operations — skills are thin wrappers that call `godmode` and act on the output.
Tasks persist in `.ctx/godmode/tasks.yaml` (gitignored) across sessions via causal
`depends_on` chains.

## Install

```bash
cargo build --release -p godmode-cli && cp target/release/godmode ~/.cargo/bin/godmode
claude plugin install godmode@bazaar
# OpenCode commands and agents are installed independently:
godmode command install-opencode --dry-run
godmode agent install-opencode --dry-run
```

### Pre-commit hook

Install the godmode pre-commit hook into any git repo to block commits when tasks are
still running and to run cargo gates (fmt-check, clippy, nextest):

```bash
# From the repo root where you want the hook installed:
nu /path/to/godmode/hooks/install.nu

# Or, if godmode is installed as a plugin and $CLAUDE_PLUGIN_ROOT is set:
nu "$CLAUDE_PLUGIN_ROOT/hooks/install.nu"
```

The hook:

1. Runs `godmode handoff --json` — exits non-zero if any task is in `running` state,
   printing the offending task IDs.
2. Runs `cargo fmt --all --check`, `cargo clippy --workspace -- -D warnings`, and
   `cargo nextest run --workspace`.
3. Blocks the commit on any failure with a clear message.

If `godmode` is not on PATH, the task-state check is skipped gracefully and only the
cargo gates run.

## Skills

| Skill                                     | When                                                    |
| ----------------------------------------- | ------------------------------------------------------- |
| `godmode:using-godmode`                   | Session orientation, available skills, rules            |
| `godmode:task-driven-development`         | Implementing any feature or fix (TDD + YAML task list)  |
| `godmode:systematic-debugging`            | Any bug, test failure, unexpected behavior              |
| `godmode:brainstorm`                      | Before any creative or design work                      |
| `godmode:writing-plans`                   | Multi-step task with a spec or requirements             |
| `godmode:ingest`                          | Load a completed plan into the task graph               |
| `godmode:verification-before-completion`  | Before claiming work is done                            |
| `godmode:task-management`                 | Tracking and executing an existing task graph           |
| `godmode:parallel-agents`                 | 2+ independent tasks to run concurrently                |
| `godmode:code-review`                     | Quality pass before merge                               |
| `godmode:refactoring`                     | Restructure code without changing behaviour             |
| `godmode:receiving-review`                | Process incoming review feedback                        |
| `godmode:cap`                             | Commit and push with validation                         |
| `godmode:ci-fix`                          | Fix a failing CI pipeline                               |
| `godmode:tackle-issues`                   | Work GitHub issues in parallel worktrees                |
| `godmode:testing-philosophy`              | Choose the right test type for the situation            |
| `godmode:introspection`                   | Audit skills and plugin files for consistency           |
| `godmode:observability-as-infrastructure` | Query and tail the session trace log                    |
| `godmode:wave-integration`                | Merge parallel agent branches into one commit           |
| `godmode:moa`                             | Multi-model reasoning via mixture of agents             |
| `godmode:todo-issue-sync`                 | Audit inline TODOs and sync to GitHub issues            |
| `godmode:self-reflect`                    | Session retrospective — patterns and surprises          |
| `godmode:decompose`                       | Break a large diff/PR into smaller independent PRs      |
| `godmode:merge`                           | Merge branches, resolve conflicts, create PRs           |
| `godmode:agent-governance`                | Governance and trust controls for AI agent systems      |
| `godmode:context-map`                     | Map all files relevant to a task before changes         |
| `godmode:doublecheck`                     | Three-layer verification of AI-generated output         |
| `godmode:rust-conventions`                | Rust coding conventions and best practices              |
| `godmode:mini-context-graph`              | Persistent knowledge graph for codebase exploration     |
| `godmode:memory-banking`                  | Generate and maintain .ctx/godmode/memory-bank/ context |

## Agents

| Agent              | Purpose                                            |
| ------------------ | -------------------------------------------------- |
| `gm-crate`         | TDD implementation in a single Rust crate          |
| `gm-tdd-helper`    | Strict TDD — failing test before implementation    |
| `gm-dispatch`      | Parallel dispatch of independent task chains       |
| `gm-wave`          | Merge parallel agent branches into main            |
| `gm-debug`         | Systematic debugging — root cause before fix       |
| `gm-brainstorm`    | Design and architecture specialist                 |
| `gm-planner`       | Convert approved designs into implementation plans |
| `gm-orchestrator`  | Planning and TDD workflow orchestrator             |
| `gm-tasker`        | Task graph management and session progress         |
| `gm-verify`        | Verification gate before completion claims         |
| `gm-refactor`      | Refactoring with strict test discipline            |
| `gm-review`        | Process incoming code review feedback              |
| `gm-code-review`   | Structured quality pass before merge               |
| `gm-cap`           | Commit and push with cargo gates                   |
| `gm-ci-fix`        | CI failure diagnosis and repair                    |
| `gm-issues`        | Dispatch GitHub issues to parallel agents          |
| `gm-orient`        | Godmode orientation and help                       |
| `gm-testing`       | Test strategy advisor (read-only)                  |
| `gm-trace`         | Session trace analysis (read-only)                 |
| `gm-introspection` | Plugin audit and conformance checks                |
| `gm-moa`           | Mixture-of-Agents synthesis                        |
| `valerie`          | Task and todo management specialist                |

## CLI Reference

### Session

```bash
godmode handon      # triage at session start — prints running, next, blocked
godmode handoff     # validate at session end — warns on tasks left running
```

### Task graph

```bash
godmode task list
godmode task add <title> [--id <id>] [--depends-on t1,t2] [--crate-name <crate>]
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

Parses `### Task N: <title>` headings, optional **Crate**: `name` and **Run**: `cmd` annotations. Builds sequential deps automatically. Idempotent —
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

`.ctx/godmode/tasks.yaml` — ephemeral, gitignored. Schema:

```yaml
tasks:
  - id: t1
    title: "Write failing test for FooAdapter"
    status: done # canonical: pending | running | done | blocked
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

`active` is accepted when reading older task graphs and normalized to `running`
on the next write. Supported Godmode commands always emit the canonical values.

## Workflow

```
brainstorm → writing-plans → ingest → task-management
  → task next → task start → [tdd] → task done → task next → ...
  → dispatch (parallel chains) → parallel-agents
  → verification-before-completion → handoff
```

## Commands

Slash commands live in `commands/`; their sources live in `command-support/gm/` and map to skills or multi-skill workflows. The planning sequence is:

```
/gm:brainstorm → /gm:design → /gm:plan → /gm:ingest → /gm:implement
```

Render and install commands for both clients from `command-support/gm/*.yaml`:

```bash
godmode command generate --target claude
godmode command install-opencode
```

**Atomic** (single skill): `cap`, `tdd`, `debug`, `refactor`, `review-code`, `ci-fix`,
`self-heal`, `brainstorm`, `design`, `plan`, `ingest`, `implement`, `tackle-issues`,
`moa-review`, `preflight`, `handon`, `handoff`,
`fresh-branch`, `introspect`, `dispatch-all`, `auth-fail-fast`, `test-fix-commit`, `trace`

**Workflow** (multi-skill pipelines):

| Command           | Pipeline                                                                    |
| ----------------- | --------------------------------------------------------------------------- |
| `ideate`          | repo scan → gap analysis → brainstorm                                       |
| `feature`         | brainstorm → design → writing-plans → ingest → tdd → verify → cap           |
| `ship`            | verification → changelog → release-notes → cap                              |
| `release`         | readiness-check → impact → bump → changelog → release-notes → cap           |
| `audit`           | health-score → dead-code → dep-audit → mistake-tracker → repo-gap-backlog   |
| `pr`              | code-review → doublecheck → pr-author → merge                               |
| `review-incoming` | receiving-review → verify → cap                                             |
| `deps`            | dep-audit → dep-bump → cap                                                  |
| `session-end`     | whatidid → self-reflect → mistake-tracker → memory-banking → session-wrap   |
| `improve-agent`   | self-reflect → pattern-learner → agent-improvement-loop → agents-skill-save |
| `context`         | context-map → memory-banking → mini-context-graph                           |

See `command-support/gm/README.md` for the full reference.

## Helpers

Helper scripts live in `skills/<skill>/helpers/`. Shared utilities are in `skills/_lib/`:

| Module       | Purpose                                                     |
| ------------ | ----------------------------------------------------------- |
| `helpers.nu` | `repo-root`, `run-checked`, `cargo-gate`, `assert-not-main` |
| `trace.nu`   | Structured JSONL trace events (skill.start/complete/error)  |

The `task-driven-development` skill ships a standalone `rust-script` helper:

| Script                                                  | Purpose                                                                           |
| ------------------------------------------------------- | --------------------------------------------------------------------------------- |
| `skills/task-driven-development/helpers/task-runner.rs` | Phase runner: init / red / green / refactor / next / status / fail / close-issues |

Trace output lands in `.ctx/godmode/traces/trace.jsonl`. Query it with
`godmode trace tail`, `godmode trace failures`, `godmode trace stats`, or
`godmode trace summary`. Use `godmode:observability-as-infrastructure`
to query it.

### Session tracing

Two hooks emit session lifecycle events automatically:

- `hooks/scripts/session-start.nu` (SessionStart) — writes `session.start` to the trace
- `hooks/scripts/stop-guard.nu` (Stop) — writes `session.end` on clean exit

Both delegate to `hooks/scripts/godmode-trace.rs` (a `rust-script` binary) which owns all
trace I/O. A fresh `session_id` (`<git-sha>-<epoch-ms>`) is generated each session and
persisted to `.ctx/godmode/session.json`. The session file is read by `session.end` to
correlate the pair. Both hooks degrade silently if not in a git repo or if `rust-script`
is unavailable.

## Development

```bash
cargo fmt --all --check
cargo clippy --workspace -- -D warnings
cargo nextest run --workspace
cargo check --workspace
```

## License

MIT OR Apache-2.0
