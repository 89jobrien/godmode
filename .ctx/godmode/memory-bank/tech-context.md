# Tech Context

## Stack and workspace

- Rust edition 2024, workspace version 0.7.0, with four members: `godmode-core`, `godmode-cli`, `tests/conformance`, and `xtask`. [Source: `Cargo.toml:1-9`]
- Clap provides CLI parsing; Serde JSON/YAML and Chrono support persisted state; Petgraph supports dependency graphs; Tokio and Rayon support concurrency; Anyhow, Thiserror, and Miette provide error layers; Slashcrux and Crux Runtime support workflow and trace integration. [Source: `Cargo.toml:13-31`]
- `godmode-core` is the public domain/integration library; `godmode-cli` is the binary presentation layer with extracted family adapters under `crates/godmode-cli/src/commands/`. [Sources/tests: `crates/godmode-core/src/lib.rs`, `crates/godmode-cli/src/main.rs:819`, `crates/godmode-cli/src/commands/mod.rs`, `crates/godmode-cli/tests/cli_boundary.rs`]

## Build, test, and quality commands

```bash
cargo build --workspace
cargo check --workspace
cargo clippy --workspace -- -D warnings
cargo nextest run --workspace
cargo fmt --all --check
cargo run -p godmode-conformance --bin run-conformance -- --verbose
just conformance
cargo xtask pre-commit
cargo xtask ci
```

[Sources: `AGENTS.md:8-24`, `xtask/src/main.rs:38-123`]

- The gate evidence inherited by functional/session baseline `3589433` records 644/644 nextest tests, clippy with warnings denied, and formatting; rustdoc and 52/52 conformance also passed. [Audit: `.ctx/godmode/review/99-resolution.md:112-123`]
- The preferred installed binary path remains `$HOME/.cargo/bin/godmode`. [Source: `AGENTS.md:27-34`]

## Runtime and persistence formats

| Contract | Format and path | Evidence |
| --- | --- | --- |
| Task graph | YAML, `.ctx/godmode/tasks.yaml` | `crates/godmode-core/src/model.rs:54-156` |
| Wave state | JSON, `.ctx/godmode/wave-status.json` | `crates/godmode-core/src/wave.rs` |
| Workflow definition/state | YAML input; JSON state at `.ctx/godmode/workflow-<name>.json` | `crates/godmode-core/src/workflow.rs` |
| Pipeline definition/state | YAML; state at `.ctx/godmode/pipeline.yaml` | `crates/godmode-core/src/pipeline.rs` |
| Governance policy | YAML under `skills/agent-governance/policies/` | `crates/godmode-core/src/policy.rs` |
| Session traces | JSONL under `.ctx/godmode/sessions/` through `SessionTracePort` | `crates/godmode-core/src/session.rs:38-49`, `crates/godmode-core/src/session.rs:76-103` |
| Insights | Canonical JSONL at `.ctx/godmode/traces/insights.jsonl`; daily Markdown under `.ctx/godmode/reports/insights/` | `crates/godmode-core/src/insights.rs:31-45`, `crates/godmode-core/src/insights.rs:89-215` |
| Report index | JSON at `.ctx/godmode/reports/godmode-reports.index.json` behind `ReportIndexPort` | `crates/godmode-core/src/report_index.rs:49-84` |

## Architectural seams in functional/session baseline `3589433`

- Session infrastructure is injectable through graph-store, run-validator, trace-sink, and clock traits collected in `SessionPorts`. [Source/tests: `crates/godmode-core/src/session.rs:23-151`, `crates/godmode-core/src/session.rs:901-950`]
- OpenCode implementation lives in `crates/godmode-core/src/agent/opencode.rs`; `agent.rs` re-exports its public surface and has a boundary test against reintroducing implementation. [Sources/tests: `crates/godmode-core/src/agent.rs:10-16`, `crates/godmode-core/src/agent.rs:322-340`, `crates/godmode-core/src/agent/opencode.rs`]
- Command rendering and filesystem mechanics have dedicated module boundaries at `crates/godmode-core/src/command/render.rs` and `crates/godmode-core/src/command/fs.rs`; `render_commands_with_templates` is the pure in-memory entry point. [Sources/tests: `crates/godmode-core/src/command.rs:9-10`, `crates/godmode-core/src/command.rs:176-204`, `crates/godmode-core/src/command.rs:572-594`]
- Insight report generation accepts `&dyn ReportIndexPort`; `JsonFileIndex` is the concrete JSON adapter. [Sources/tests: `crates/godmode-core/src/insights.rs:176-215`, `crates/godmode-core/src/insights.rs:359-379`, `crates/godmode-core/src/report_index.rs:49-218`]

## Command execution and compatibility constraints

- `run:` values beginning with `rx:` invoke the registry; plain strings split into program and args; shell metacharacters invoke the detected shell with `-c`. [Source: `crates/godmode-core/src/integrations/rx.rs:13-55`]
- Missing `rx` skips registry validation, but an unknown registered script is rejected when `rx` is available; `Session` routes this policy through `RunValidatorPort`. [Sources: `crates/godmode-core/src/integrations/rx.rs:72-115`, `crates/godmode-core/src/session.rs:28-32`, `crates/godmode-core/src/session.rs:244-254`]
- Task status accepts legacy `active` and emits canonical `running`; insight migration similarly preserves legacy history while moving storage to canonical paths. [Sources/tests: `crates/godmode-core/src/model.rs:54-83`, `crates/godmode-core/src/insights.rs:89-139`, `crates/godmode-core/src/insights.rs:313-335`]
