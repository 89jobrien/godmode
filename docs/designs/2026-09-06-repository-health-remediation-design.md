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

The remediation adds and preserves these production APIs:

- `godmode_core::command`: `CommandTarget`, `CommandDefinition`, `RenderedCommand`,
  `load_command_definitions`, `render_commands`, `write_rendered_commands`, and
  `check_rendered_commands`.
- `godmode_core::report_index`: `ReportIndex`, `ReportCategory`, `ReportIndexPort`, and
  `JsonFileIndex` for incremental updates and on-disk reconciliation.
- `godmode_core::trace_stats`: `SkillDuration`, `AgentConvergence`, `TraceStats`,
  `TraceSessionSummary`, and the `tail`, `failures`, `stats`, `summaries`, and
  `current_session_id` query functions.

Existing feature-gated testing APIs remain compatible. Potential removals such as `GateStep` and
compatibility-only feature flags remain deferred to a breaking release with issue-linked TODOs.

### Serialized status and governance contracts

- Task state serializes as `pending`, `running`, `done`, or `blocked`; legacy `active` remains an
  accepted input alias but is never emitted. `StatusCache` exposes the matching `running` count
  alongside `pending`, `blocked`, `done`, `project`, and `updated_at`.
- Governance actions serialize as `allow`, `deny`, or `review`, and levels as `open`, `standard`,
  `strict`, or `locked`. Policy YAML includes inheritance, tool allow/block lists, blocked input
  patterns, call limits, approval requirements, subagent constraints, and audit settings.
- Resolved-policy JSON includes `policy`, `agent`, `category`, `level`, and ordered `sources`;
  tool checks include `action` and `reason`. These fields are consumer-facing schema, not prose-only
  implementation details.

## Data Flow

1. Canonical command YAML and templates flow through `godmode-core::command` into deterministic
   Claude and OpenCode projections.
2. Canonical agent cfg and prompt files flow through the agent generator into generated Markdown
   and the deduplicated agent index.
3. Cargo workspace version metadata flows through release validation into both plugin manifests;
   hooks validate parity rather than stamping commit hashes.
4. Task state flows through canonical status serialization and `StatusCache`; the Rust
   pre-commit checker and Nu adapters reject unresolved running or blocked tasks.
5. Governance YAML composes into resolved-policy JSON and allow/deny/review audit events consumed
   by CLI and hook adapters.
6. `xtask` composes formatting, all-target/all-feature clippy, workspace tests, conformance,
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
- Reverting or replacing the existing staged command-renderer and OpenCode migration.

## Risk

- [x] Breaking API changes: no; candidates are deferred with issue-linked TODOs.
- [x] New external dependency: no.
- [x] Feature flag required: existing `testing` and compatibility flags are preserved.
- [x] Persisted state changes: canonical task/status output uses `running`; legacy `active` remains input-compatible, and governance/status JSON fields are documented above.
- [x] CLI output changes: generated command output and release validation become stricter; human and
      JSON contracts receive integration coverage.
