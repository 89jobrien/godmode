# System Patterns

## Functional/session baseline `3589433`

- The workspace remains four members: `godmode-core`, `godmode-cli`, `tests/conformance`, and `xtask`. [Source: `Cargo.toml:1-4`]
- `godmode-core` owns domain policy and ports; `godmode-cli` parses arguments and delegates command-family behavior through `commands::dispatch`. [Sources/tests: `crates/godmode-core/src/lib.rs`, `crates/godmode-cli/src/main.rs:819`, `crates/godmode-cli/src/commands/mod.rs`, `crates/godmode-cli/tests/cli_boundary.rs`]
- CLI family handlers are separated into `crates/godmode-cli/src/commands/{agent,command,session,trace,workspace}.rs`; hook and task handlers retain their focused adapters. [Sources: `crates/godmode-cli/src/commands/agent.rs`, `crates/godmode-cli/src/commands/command.rs`, `crates/godmode-cli/src/commands/session.rs`, `crates/godmode-cli/src/commands/trace.rs`, `crates/godmode-cli/src/commands/workspace.rs`, `crates/godmode-cli/src/commands/hook.rs`, `crates/godmode-cli/src/commands/task.rs`; Audit: `.ctx/godmode/review/99-resolution.md:50-53`]

## Session ports and lifecycle

1. `SessionPorts` injects `TaskGraphStorePort`, `RunValidatorPort`, `SessionTracePort`, and `ClockPort`; defaults adapt the graph file, rx validation, JSONL traces, and UTC system time. [Source: `crates/godmode-core/src/session.rs:23-151`]
2. `Session::open_with_config_and_ports` loads through the graph-store port. Start validates through the run-validator before mutation, transitions use the clock, trace through the sink, and persistence routes through the store. [Source: `crates/godmode-core/src/session.rs:188-343`]
3. Trace writes and auto-save preserve their best-effort behavior, while explicit `save` and summary writes return errors. [Source: `crates/godmode-core/src/session.rs:323-354`]
4. `session_uses_injected_run_validator_without_breaking_facade` and `session_routes_transition_and_summary_time_through_clock_port` verify the seams and deterministic timing. [Tests: `crates/godmode-core/src/session.rs:901-950`; Audit: `.ctx/godmode/review/99-resolution.md:52`]

## Agent and command boundaries

- Generic agent definitions remain in `agent.rs`; OpenCode catalog types, validation, rendering, transactional installation, and filesystem mechanics are owned by `agent::opencode` and re-exported for compatibility. [Sources: `crates/godmode-core/src/agent.rs:10-16`, `crates/godmode-core/src/agent/opencode.rs:12-522`]
- `generic_agent_module_does_not_implement_opencode_catalog` prevents OpenCode policy from leaking back into the generic facade. Transaction failure tests cover per-file failure, restoration, and stale artifacts. [Tests: `crates/godmode-core/src/agent.rs:322-340`, `crates/godmode-core/src/agent/opencode.rs:635-680`; Audit: `.ctx/godmode/review/99-resolution.md:53`, `.ctx/godmode/review/99-resolution.md:56`]
- The command facade declares `command::fs` and `command::render`; filesystem loading/writing and pure target rendering are separate boundaries while compatibility APIs remain in `command.rs`. [Sources: `crates/godmode-core/src/command.rs:9-10`, `crates/godmode-core/src/command/fs.rs`, `crates/godmode-core/src/command/render.rs`]
- `render_commands_with_templates` accepts preloaded templates and performs no filesystem access; pure rendering and missing-template behavior have direct tests. [Source/tests: `crates/godmode-core/src/command.rs:176-204`, `crates/godmode-core/src/command.rs:572-594`; Audit: `.ctx/godmode/review/99-resolution.md:54`]

## Insight migration and report indexing

- Insight listing reads canonical `.ctx/godmode/traces/insights.jsonl` and legacy `.ctx/insights.jsonl`, skips malformed rows, deduplicates records, writes the merged canonical store, and removes the legacy JSONL after successful migration. [Source/test: `crates/godmode-core/src/insights.rs:89-139`, `crates/godmode-core/src/insights.rs:313-335`; Audit: `.ctx/godmode/review/99-resolution.md:22-23`]
- Daily Markdown belongs under `.ctx/godmode/reports/insights/`; legacy `.ctx/insights-YYYY-MM-DD.md` is moved or copied before cleanup. [Source: `crates/godmode-core/src/insights.rs:176-215`]
- `ReportIndexPort` defines add-entry, add-item, rebuild, and load behavior; `JsonFileIndex` is the filesystem adapter. Insight rendering accepts the port and treats index updates as best effort. [Sources: `crates/godmode-core/src/report_index.rs:49-65`, `crates/godmode-core/src/report_index.rs:70-218`, `crates/godmode-core/src/insights.rs:176-215`]
- `render_markdown_with_index_uses_injected_port_and_migrates_legacy_report` verifies both migration and injected index behavior. [Test: `crates/godmode-core/src/insights.rs:359-379`; Audit: `.ctx/godmode/review/99-resolution.md:55`]

## Persistence and trust contracts

- Task status input accepts legacy `active` while canonical serialization emits `running`; task graph state remains YAML at `.ctx/godmode/tasks.yaml`. [Source: `crates/godmode-core/src/model.rs:54-156`]
- Wave, workflow, pipeline, governance, insight, trace, and report-index formats are persisted compatibility boundaries. [Sources: `crates/godmode-core/src/wave.rs`, `crates/godmode-core/src/workflow.rs`, `crates/godmode-core/src/pipeline.rs`, `crates/godmode-core/src/policy.rs`, `crates/godmode-core/src/insights.rs`, `crates/godmode-core/src/session.rs`, `crates/godmode-core/src/report_index.rs`]
- Workflow steps can resolve directly to process execution or a shell; workflow definitions remain trusted executable input. [Sources: `crates/godmode-core/src/workflow.rs:121-169`, `crates/godmode-core/src/integrations/rx.rs:13-55`]

## Patterns to preserve

- Add infrastructure behavior behind focused ports/adapters while retaining documented compatibility facades. [Sources: `crates/godmode-core/src/session.rs:23-151`, `crates/godmode-core/src/agent.rs:10-16`, `crates/godmode-core/src/command.rs:9-10`]
- Keep pure rendering separate from filesystem mutation and use transactional projection/install paths. [Sources: `crates/godmode-core/src/command.rs:176-204`, `crates/godmode-core/src/projection.rs`, `crates/godmode-core/src/agent/opencode.rs:257-381`]
- Keep CLI `main.rs` limited to parsing and delegation; enforce the boundary with `crates/godmode-cli/tests/cli_boundary.rs`. [Source/test: `crates/godmode-cli/src/main.rs:819`, `crates/godmode-cli/tests/cli_boundary.rs`]
- Audit a fixed committed baseline independently from preserved local state before claiming remediation complete. [Audit: `.ctx/godmode/review/99-resolution.md:1-6`, `.ctx/godmode/review/99-resolution.md:90-123`]
