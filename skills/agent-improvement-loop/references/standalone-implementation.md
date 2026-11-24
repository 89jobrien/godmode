# Standalone implementation

Read this before installing or running the standalone updog workspace or its ail binary.

## Standalone implementation

The loop is also available as a standalone Rust workspace: **updog** (`~/dev/updog`,
<https://github.com/89jobrien/updog>).

- `agent-loop` crate — serializable types: `TraceRecord`, `FeedbackCluster`, `HaloScore`,
  `ChangeItem`, `Diagnosis`, `Handoff`
- `ail` binary — `ail run [--agent <name>] [--since <days>] [--phase <N>] [--dry-run]`

Install: `cargo install --path ~/dev/updog/crates/ail`

Phases 1, 4, 5 are fully implemented. Phases 2, 3 emit guided prompts and resume instructions.
Phases 6, 7 print next steps and run `crs validate`.
