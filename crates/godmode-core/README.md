# `godmode-core`

<!-- markdownlint-disable MD013 -->

`godmode-core` is the library crate behind the `godmode` command. It owns the task graph and
session domain model, repository state files, plan and template ingestion, verification and review
engines, agent and workflow metadata, governance policy evaluation, and adapters for external
tools. `godmode-cli` is intentionally the presentation layer over these APIs.

The crate follows the workspace version and Rust 2024 edition. It is exposed as the Rust library
`godmode_core`; doctests are disabled. Most APIs are synchronous, while async dispatch support uses
Tokio.

## Workspace Role

The workspace separates responsibilities as follows:

| Package               | Role                                                                    |
| --------------------- | ----------------------------------------------------------------------- |
| `godmode-core`        | Domain types, state transitions, persistence, integrations, and reports |
| `godmode-cli`         | Clap command model, human/JSON/SARIF rendering, process exit behavior   |
| `godmode-conformance` | Cross-module behavioral contracts, fixtures, properties, reports        |
| `xtask`               | Repository quality, distribution, and installation commands             |

Consumers can use `godmode-core` without depending on Clap or CLI output formatting.

## Core Task Model

The primary types are in `model`:

- `TaskGraph` contains ordered `Task` values and private plan-ingestion provenance.
- `Task` records its ID, title, status, dependencies, notes, external provenance, optional crate,
  run command, commit, dates, timings, priority, and tags.
- `Status` serializes as `pending`, `running`, `done`, or `blocked`.
- `Priority` serializes as `high`, `normal`, or `low`; `normal` is the default and is omitted from
  YAML.
- `TaskProvenance` currently carries an optional structured Doob ID.

`graph` provides the low-level operations:

```rust
use godmode_core::{graph, model::{Task, TaskGraph}};

let mut tasks = TaskGraph::default();
graph::add(&mut tasks, Task::new("t1", "Write the contract test"))?;
graph::start(&mut tasks, "t1")?;
graph::complete(&mut tasks, "t1", Some("abc123"), None)?;
assert!(graph::runnable(&tasks).is_empty());
# Ok::<(), anyhow::Error>(())
```

`graph::add` rejects duplicate IDs and dependency cycles. `graph::start` only accepts pending tasks
whose dependencies are done. `graph::complete` only accepts running tasks. Removal also deletes
references to the removed task, while `clear` can remove only done tasks or the complete graph.
`to_petgraph` and `to_dot` expose graph projections for analysis and visualization.

## Session API

`session::Session` is the preferred stateful facade. It loads configuration and the task graph,
performs typed transitions, persists transition state, validates `rx:` run commands when enabled,
and emits Crux-compatible JSONL events when configured.

```rust
use godmode_core::{model::Task, session::Session};

let root = std::path::Path::new("/path/to/repository");
let mut session = Session::open(root)?;
session.add_task(Task::new("t1", "Document the public API"))?;
session.save()?;
session.start_task("t1")?;
session.complete_task("t1", None, Some("README updated"))?;
# Ok::<(), anyhow::Error>(())
```

Important behavior:

- `Session::start_task` sets `started_at`; direct `graph::start` does not.
- `Session::complete_task` calculates duration, records completion metadata, and auto-saves.
- `Session::ingest_plan` restores the in-memory graph if persistence fails.
- `Session::summary` returns status counts and per-task durations.
- `handon` and `handoff` are compatibility helpers for session-start and session-end reporting.

Use `Session::graph()` for reads. `graph_mut()` is a hidden escape hatch; direct mutation bypasses
the auto-save and trace invariants maintained by typed session methods.

## Plans, Templates, And Dispatch

`plan::parse` recognizes `### Task N: Title` or `### N. Title` headings and these optional
annotations:

```markdown
### Task 1: Add a failing parser test

**Crate**: `godmode-core`
**Run**: `cargo nextest run -p godmode-core`
**Depends-on**: `graph:existing-task`
```

Without `**Depends-on**`, parsed tasks form a sequential chain. `plan::ingest` keeps the first
available `tN` IDs, deterministically namespaces collisions from the source filename, rewrites
internal dependencies, preserves explicit `graph:` dependencies, and records source-to-ID mappings
for idempotent re-ingestion. The recorded source path is one identity signal, not the whole rule: a
plan whose recorded path no longer exists is matched as relocated when its file stem is unambiguous
and its recorded tasks are still in the graph with matching title, crate, and run, so re-ingestion
stays idempotent after a move. Two plan files that share a stem and both still exist stay distinct.
`IngestReport` reports parsed, added, skipped, and assigned IDs.

`templates` loads task templates from repository `templates/` before
`$HOME/.config/godmode/templates/`, substitutes declared `{{variables}}`, and applies resolved tasks
to a graph. Template parse and substitution failures use `miette` diagnostics.

`dispatch` provides:

- `independent_chains` for bounded, crate-oriented task chains.
- `critical_path` for the longest active dependency chain.
- `dispatch_with_config` and `async_dispatch_with_config` for retry-aware execution.
- `file_overlap_warnings` for chains that target the same crate.

## Other Public Subsystems

| Modules                                      | Responsibility and representative API                                        |
| -------------------------------------------- | ---------------------------------------------------------------------------- |
| `config`, `detect`                           | Config precedence, project naming, repository discovery, root pinning        |
| `pipeline`                                   | YAML skill pipelines, persisted step history, optional and parallel steps    |
| `workflow`                                   | Agent workflow DAG parsing, runnable-step selection, resumable execution     |
| `wave`, `worktree`                           | Parallel slot state, retry settings, and guarded Git worktree lifecycle      |
| `verify`                                     | Pluggable `VerifyStep` gates and aggregate `VerifyReport` values             |
| `review`                                     | Skill, agent, plugin manifest, naming, and shared-library conformance checks |
| `policy`                                     | Layered governance policy resolution, tool checks, and JSONL audit events    |
| `agent`, `agent_index`                       | Agent generation, discovery, filtering, and index generation                 |
| `skill`, `registry`                          | Local skill installation and the global artifact registry                    |
| `eval`                                       | Bounded skill evaluations, run persistence, baselines, comparison, CI gates  |
| `context`, `cache`                           | Hook/subagent context and a fast status cache for prompt integrations        |
| `memory_banking`, `insights`, `report_index` | Persistent project context and reports                                       |
| `hooks`, `integrations`                      | Built-in hooks and adapters for GitHub, Doob, Hj, Rx, and Crux               |
| `release`                                    | Version bumping, tags, changelog generation, and version validation          |
| `sarif`, `scaffold`, `test_check`            | Machine-readable findings and testing support commands                       |

External command integrations are adapters rather than hard runtime dependencies. Their callers
generally degrade gracefully when an optional tool is unavailable; command-specific modules define
the exact fallback behavior.

## Persistence And Serialization Contracts

The crate uses Serde-backed YAML, JSON, TOML, and JSONL contracts. The main paths are:

| Path                                  | Format               | Owner                            |
| ------------------------------------- | -------------------- | -------------------------------- |
| `.ctx/godmode/tasks.yaml`             | YAML                 | `model`, `graph`, `session`      |
| `.ctx/GODMODE.tasks.yaml`             | legacy YAML fallback | `graph`                          |
| `.ctx/godmode/session.json`           | JSON                 | root pinning and session hooks   |
| `.ctx/godmode/sessions/*.jsonl`       | JSONL                | task steps and session summaries |
| `.ctx/godmode/pipeline.yaml`          | YAML                 | active pipeline state            |
| `.ctx/godmode/wave-status.json`       | JSON                 | parallel wave slots              |
| `.ctx/godmode/workflow-<name>.json`   | JSON                 | workflow execution state         |
| `.ctx/godmode/evaluations/<skill>/`   | JSON                 | evaluation runs and baseline     |
| `$HOME/.config/godmode/registry.json` | JSON                 | installed skills and agents      |

`graph::load` returns an empty graph when no task file exists. New writes use
`.ctx/godmode/tasks.yaml`; the legacy file is read only when the new path is absent. Optional and
defaulted task fields preserve compatibility with older task files. Internal done-task caches are
never serialized.

Configuration loads in this order:

1. `<repo>/.godmode.toml`
2. `$HOME/.config/godmode/config.toml`
3. built-in defaults

The default integration settings enable Doob, Hj, Crux, and Rx, while Coursers validation is off.
The handoff writer and Doob handoff synchronization are enabled with a ten-commit limit.

## Features

| Feature      | Effect                                                              |
| ------------ | ------------------------------------------------------------------- |
| default      | Runtime library only                                                |
| `testing`    | Enables `testing::{audit,binary,conformance,env,prop,seed}` helpers |
| `test-utils` | Alias that enables `testing`                                        |

The workspace conformance package consumes the ordinary public runtime API. The feature-gated
`testing` module is reusable support for adapter suites, dependency/snapshot audits, isolated test
environments, property assertions, deterministic seeds, and binary lookup.

## Development And Testing

From the workspace root:

```console
cargo check -p godmode-core --all-features
cargo clippy -p godmode-core --all-targets --all-features -- -D warnings
cargo nextest run -p godmode-core --all-features
cargo run -p godmode-conformance --bin run-conformance -- --verbose
```

Use temporary directories in tests for all persisted state. When changing serialized types, add a
round-trip or compatibility test and update the conformance fixtures when the contract change is
intentional. When adding a public subsystem, export it from `src/lib.rs` and keep the CLI layer
focused on argument parsing and rendering.
