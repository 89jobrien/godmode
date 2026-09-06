# Design: Repository Health Remediation

## Goal

Resolve every actionable finding from the repository health audit while preserving the
in-progress command-renderer migration and recording only genuinely deferred breaking work as
issue-linked code TODOs.

## Approved Approach

Apply systemic fixes by contract boundary, regenerate derived artifacts from their authoritative
sources, and add TODO comments only where compatibility or external dependency constraints prevent
completion in this pass.

## Crate Ownership

- **`godmode-core`** owns task-state enforcement, release-version validation, command rendering,
  index validation, and reusable testing helpers.
- **`godmode-cli`** remains a thin adapter and gains integration coverage for command rendering.
- **`godmode-conformance`** owns repository-wide command, plugin, agent, and generated-output
  contracts.
- **`xtask`** remains the single local/CI quality-gate entry point.
- Repository hooks, manifests, documentation, and GitHub Actions are adapters around those Rust
  contracts and must not define competing behavior.

## Public API

No new production public API is required. Existing feature-gated testing APIs remain compatible and
receive missing documentation. Potential removals such as `GateStep` and compatibility-only feature
flags are deferred to a breaking release and receive issue-linked TODO comments.

## Data Flow

1. Canonical command YAML and templates flow through `godmode-core::command` into deterministic
   Claude and OpenCode projections.
2. Canonical agent cfg and prompt files flow through the agent generator into generated Markdown
   and the deduplicated agent index.
3. Cargo workspace version metadata flows through release validation into both plugin manifests;
   hooks validate parity rather than stamping commit hashes.
4. Task state flows through the Rust pre-commit checker; Nu hooks delegate to that contract and
   reject unresolved running or blocked tasks.
5. `xtask` composes formatting, all-target/all-feature clippy, workspace tests, conformance,
   generated-artifact checks, hook parsing, and dependency policy for local and CI use.

## Integration Points

- Complete the staged Markdown-to-JSON skill-index migration in Rust and Nu conformance checks.
- Validate agent command examples against exact Clap command paths and supported flags, then
  regenerate agents from source prompts.
- Expand command-renderer unit, CLI, and cross-target conformance coverage before changing write or
  collision behavior.
- Synchronize README, CLAUDE, AGENTS, pipeline documentation, plan statuses, manifests, and license
  files with executable behavior.
- Upgrade direct dependencies in isolated batches; retain unavoidable transitive duplicate versions.

## Deferred TODO Policy

- Every deferred code TODO must cite an open GitHub issue and state the compatibility constraint.
- No TODO is added for work completed in this remediation.
- No TODO is added merely to mirror documentation, metadata, or dependency-update notes.
- `getrandom` and `unicode-width` duplication remains dependency-owned and is documented in the
  dependency policy rather than source comments.

## Out of Scope

- Removing public APIs or compatibility feature flags before a breaking release.
- Forcing incompatible transitive dependency convergence.
- Rewriting historical examples in completed design and plan documents.
- Reverting or replacing the existing staged command-renderer and OpenCode migration.

## Risk

- [x] Breaking API changes: no; candidates are deferred with issue-linked TODOs.
- [x] New external dependency: no.
- [x] Feature flag required: existing `testing` and compatibility flags are preserved.
- [x] Persisted state changes: no task or trace schema changes.
- [x] CLI output changes: generated command output and release validation become stricter; human and
      JSON contracts receive integration coverage.
