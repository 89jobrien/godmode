# Integrations

Subprocess wrappers for external CLI tools. All live under
`crates/godmode-core/src/integrations/`.

## Design pattern

Every integration invokes an external binary via `std::process::Command`.
Callers use `.ok()` or `.ok().flatten()` — missing tools never abort.
Each integration is toggled via `Config.integrations` booleans.

## Components

| Integration    | Binary | Purpose                                             |
| -------------- | ------ | --------------------------------------------------- |
| `cruxx`        | cruxx  | Write Step JSONL traces to `.ctx/godmode/sessions/` |
| `doob`         | doob   | Sync todos, pull pending items, handoff sync        |
| `hj`           | hj     | Read/write HANDOFF.yaml files                       |
| `rx`           | rx     | Dispatch scripts, validate `run:` commands          |
| `gh`           | gh     | CI triage (`ci.rs`), GitHub issues (`issues.rs`)    |
| `hook_runner`  | —      | Execute hook scripts                                |
| `hook_migrate` | —      | Migrate legacy hooks                                |
| `handoff_yaml` | —      | Write native HANDOFF YAML from task state           |
| `subprocess`   | —      | Shared `run()` helper for Command execution         |
| `output`       | —      | GraphOut, HandonOutput, HandoffOutput types         |

## handon / handoff

The top-level `integrations::handon()` and `integrations::handoff()` functions
orchestrate the full session-start and session-end sequences, combining
graph state, hj output, doob todos, and dirty file detection.

## Defined in

`crates/godmode-core/src/integrations/mod.rs`
