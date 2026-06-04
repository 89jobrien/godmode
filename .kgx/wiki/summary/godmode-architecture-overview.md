# Godmode Architecture Overview

Godmode is a two-crate Rust workspace (v0.6.0) for agentic task management,
deployed as a Claude Code plugin via bazaar.

## Workspace Structure

| Crate                 | Role                                       |
| --------------------- | ------------------------------------------ |
| `godmode-core`        | Library — all domain logic, 27+ modules    |
| `godmode-cli`         | Binary — thin clap CLI calling into core   |
| `godmode-conformance` | Test suite — property tests and benchmarks |

## Core Data Model

The central types live in `model`:

- **Task** — id, title, status, depends_on, crate_name, commit, run, priority, tags
- **TaskGraph** — Vec<Task> with a RefCell done_cache for fast lookup
- **Status** — Pending | Running | Done | Blocked
- **Priority** — High | Normal | Low (Normal skipped in YAML serialization)

State file: `.ctx/godmode/tasks.yaml` (ephemeral, gitignored).

## Session Lifecycle

`Session` is the central orchestrator:

1. `Session::open()` loads TaskGraph via `graph::load`
2. `Session::start_task()` validates via `rx`, sets `started_at`, writes cruxx trace
3. `Session::complete_task()` records commit SHA, `completed_at`, duration
4. `Session::handoff()` writes SessionSummary to JSONL traces

All state transitions go through Session, not raw graph functions.

## Integration Pattern

External tools (cruxx, doob, hj, rx, gh) are wrapped as subprocess calls in
`integrations/`. Every integration fails gracefully via `.ok()` — missing tools
never abort a session. Controlled by `Config.integrations` toggles.

## Key Dependencies

- `petgraph` — graph algorithms for dependency resolution
- `slashcrux` — slash-command parsing
- `crux-runtime` — agentic runtime trace model
- `tokio` — async runtime for dispatch concurrency

See [[Modules Index]] for per-module details.
