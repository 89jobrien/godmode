# Task-Graph Parser Rejects Caller-Provided `active` Status

## Observed behavior

From the `obfsck` project, task-graph operations fail when the graph contains
`tasks[15].status: active`. The installed Godmode CLI/parser rejects that value with:

```text
tasks[15].status: unknown variant active; expected pending, running, done, blocked
```

The invalid value was introduced by a caller using `active`, while the installed
CLI/parser accepts `running`. This report does not identify or claim a root-cause commit.

## Reproduction

Run these commands from the affected project, using the affected task graph and plan:

```bash
godmode handon
godmode task next --json
godmode plan ingest <plan>
```

Each command attempts to load the incompatible graph and encounters the parser error above.

## Expected behavior

The task-graph parser and callers should use one documented, compatible status vocabulary.
Graphs produced by supported callers should load successfully, and valid graph operations
should proceed without a deserialization error.

## Impact

All graph operations are blocked while the invalid status remains in the graph, including
session triage, selecting the next task, and ingesting a plan.

## Likely cause

The caller and installed CLI/parser use different status vocabularies: the caller emitted
`active`, but the parser's schema defines `running` as the corresponding status. This is a
schema/status vocabulary mismatch; the precise introducing change is unverified.

## Acceptance criteria

- [x] Establish and document the canonical task status vocabulary, including the handling of
      `active` versus `running`.
- [x] Ensure supported callers emit a status accepted by the installed CLI/parser.
- [x] Decide and implement compatibility behavior for existing graphs containing `active`
      (for example, a safe migration or an explicitly supported alias).
- [x] Verify `godmode handon`, `godmode task next --json`, and
      `godmode plan ingest <plan>` complete successfully against the affected graph.
- [x] Add regression coverage proving the chosen compatibility behavior and preserving the
      existing `pending`, `running`, `done`, and `blocked` statuses.

## Resolution

The canonical persisted vocabulary remains `pending`, `running`, `done`, and `blocked`.
The parser accepts `active` as a compatibility alias for `running`; subsequent writes
normalize the value to `running`. CLI integration coverage exercises all three affected
commands against a graph containing the alias.

Task-driven-development guidance and its helper now emit the canonical vocabulary. The helper
also reads its former `active` and `failed` values as `running` and `blocked`, respectively, and
normalizes them on save. Plan ingestion continues to create tasks as `pending`.

## Validation

- Focused core status round-trip tests: 2 passed.
- Focused CLI regression tests for `handon`, `task next --json`, plan ingestion, and producer
  guidance: 2 passed.
- Standalone task-runner compatibility test: 1 passed.
- `cargo fmt --all --check`: passed.
- `cargo clippy --workspace -- -D warnings`: passed.
- `cargo nextest run --workspace`: 569 passed.
- `just conformance`: 665 checks passed.
