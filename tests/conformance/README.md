# Godmode Conformance Package

<!-- markdownlint-disable MD013 -->

`godmode-conformance` is the workspace's non-publishable behavioral contract package. It tests
`godmode-core` as a consumer would, checks repository/plugin structure that ordinary crate unit
tests cannot cover, provides golden JSON fixtures, and benchmarks selected graph and plan hot
paths.

This package complements rather than replaces normal tests:

- `cargo nextest run --workspace --all-features` runs Rust unit, integration, and property tests.
- `run-conformance` executes registered typed cases through a purpose-built runner and report
  generator.
- `plugin-structure.nu` is the original Nushell structural checker retained alongside its typed
  Rust port.
- Criterion benchmarks track important operations without making performance assertions in CI.

Both `cargo xtask pre-commit` and `cargo xtask ci` invoke the typed conformance runner explicitly.

## Contents

- [Run The Suite](#run-the-suite)
- [What The Package Covers](#what-the-package-covers)
- [Harness API](#harness-api)
- [Report Contracts](#report-contracts)
- [Fixtures](#fixtures)
- [Adding A Conformance Case](#adding-a-conformance-case)
- [Benchmarks](#benchmarks)
- [Development Notes](#development-notes)

## Run The Suite

Run the registered typed cases with detailed text output:

```console
cargo run -p godmode-conformance --bin run-conformance -- --verbose
```

Supported runner options are parsed directly by `run_conformance.rs`:

| Option          | Behavior                                  |
| --------------- | ----------------------------------------- |
| `--verbose`     | Show passing and failing case details     |
| `--summary`     | Suppress per-case text output             |
| `--filter TEXT` | Run cases whose test name contains `TEXT` |
| `--json`        | Emit the versioned JSON report contract   |
| `--ci`          | Emit GitHub Actions workflow commands     |

Setting the `CI` environment variable also selects GitHub Actions output. The process exits 1 when
the summary contains a failure and 0 when every executed case passed or was skipped.

Run all Rust tests, including property suites and harness tests:

```console
cargo nextest run -p godmode-conformance
```

Run the legacy Nushell structural checks directly:

```console
nu tests/conformance/plugin-structure.nu
```

Run benchmarks:

```console
cargo bench -p godmode-conformance
```

## What The Package Covers

`src/lib.rs::all_tests()` registers the typed conformance modules used by the custom runner:

| Module                     | Contract area                                                  |
| -------------------------- | -------------------------------------------------------------- |
| `graph_tests`              | Graph state transitions and runnable-task behavior             |
| `hook_registry_tests`      | Built-in hook registration contracts                           |
| `plan_tests`               | Markdown plan parsing and task projection                      |
| `dispatch_tests`           | Independent chain decomposition and dispatch projections       |
| `wave_tests`               | Persisted parallel wave state and transitions                  |
| `command_projection_tests` | Command/output projections shared across layers                |
| `crux_tests`               | Crux trace/event compatibility                                 |
| `fixture_tests`            | Golden fixture loading and expected serialized values          |
| `plugin_structure_tests`   | Skills, agents, plugin metadata, links, and shared conventions |

Additional `#[test]` and proptest modules run through Cargo/nextest rather than `all_tests()`:
`property_tests`, `session_property_tests`, `wave_property_tests`, and
`pipeline_property_tests`. This distinction is intentional: the typed runner has stable report
output, while property tests use the standard Rust test harness and proptest regression support.

The plugin structure suite is repository-level coverage. Its typed Rust cases check, among other
contracts:

- every skill directory has `SKILL.md` and valid naming metadata;
- the skill index, directories, and `using-godmode` documentation agree;
- referenced `references/`, `helpers/`, and `skills/_lib/*.nu` paths exist;
- `.claude-plugin/plugin.json` uses its allowed schema;
- documented Godmode commands are recognized;
- shared merge, concurrency, blocking, and branch-guard rules stay consistent;

Standard Rust tests in the same module also check workspace/plugin version alignment, declared
license files, structured skill-index parsing, and the supported Criterion version.

## Harness API

The harness is public within the package so new suites can compose it:

- `ConformanceTest` identifies a case by name, crate, and `TestCategory`, then runs it with a
  mutable `TestContext`.
- `TestResult` is a tagged `Pass`, `Fail { reason }`, or `Skipped { reason }` result.
- `TestRunner` registers boxed or concrete cases, filters by crate/category/name, and runs in
  parallel by default through Rayon.
- `TestContext` records structured input/expected/actual values and produces unified assertion
  failures.
- `ReportGenerator` emits text, JSON, or GitHub Actions annotations from a `TestSummary`.

A minimal case follows the existing module pattern:

```rust
use godmode_conformance::harness::{
    ConformanceTest, TestCategory, TestContext, TestResult,
};

struct EmptyGraphHasNoRunnableTasks;

impl ConformanceTest for EmptyGraphHasNoRunnableTasks {
    fn name(&self) -> &str { "empty_graph_has_no_runnable_tasks" }
    fn crate_name(&self) -> &str { "godmode-core" }
    fn category(&self) -> TestCategory { TestCategory::EdgeCase }

    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let graph = godmode_core::model::TaskGraph::default();
        ctx.assert_eq(&0, &godmode_core::graph::runnable(&graph).len());
        ctx.result()
    }
}
```

`TestRunner::filter_name` performs a case-sensitive substring match on `name()`. Crate and category
filters are available in the library even though the current `run-conformance` binary exposes only
the name filter.

## Report Contracts

Text reports group results by `crate_name`, show pass/fail/skip totals, and optionally include case
details and timings. GitHub Actions mode emits `::error` annotations for failures and a final
notice/error summary.

JSON mode emits:

```json
{
  "report_version": "1.0",
  "generated_at": "<RFC3339 timestamp>",
  "summary": {
    "total": 0,
    "passed": 0,
    "failed": 0,
    "skipped": 0,
    "duration_ms": 0,
    "success": true
  },
  "results": []
}
```

Each result includes its stable `crate_name::name` ID, category, tagged status data, and duration.
Treat `report_version` as the compatibility boundary for report consumers.

## Fixtures

Golden JSON files live in `fixtures/expected/`. `FixtureLoader::new()` resolves this directory from
the conformance package's `CARGO_MANIFEST_DIR`; `FixtureLoader::with_dir()` supports isolated unit
tests. A fixture can expose arbitrary JSON plus convenience accessors for string and string-array
fields.

Current fixtures cover plan parsing and runnable graph output:

```text
fixtures/expected/plan_parse.json
fixtures/expected/graph_runnable.json
```

When a serialized contract changes intentionally, update the implementation and its fixture test
together. Do not regenerate expected output without reviewing the semantic diff.

## Adding A Conformance Case

1. Add or extend a module in `src/` and implement `ConformanceTest` for each case.
2. Return the module's cases from an `all() -> Vec<Box<dyn ConformanceTest>>` function.
3. Export the module from `src/lib.rs`.
4. Register its `all()` result in `src/lib.rs::all_tests()`.
5. Add fixtures under `fixtures/expected/` only when the contract is serialized output.
6. Run the filtered typed case, package tests, and full conformance runner.

Example validation loop:

```console
cargo run -p godmode-conformance --bin run-conformance -- --filter empty_graph --verbose
cargo nextest run -p godmode-conformance
cargo run -p godmode-conformance --bin run-conformance -- --verbose
```

Property-only tests do not need registration in `all_tests()`; export their module from `lib.rs` so
the standard Rust test harness discovers them.

## Benchmarks

`benches/conformance_bench.rs` currently measures:

- `graph::runnable` on linear chains;
- repeated `graph::add` operations;
- `graph::unblock_all` on blocked graphs;
- `plan::parse` across plan sizes;
- `dispatch::independent_chains` across flat graphs.

Criterion HTML reports are enabled. Keep benchmark setup deterministic and outside the measured
operation where possible.

## Development Notes

- Keep tests isolated with `tempfile`; repository state must not leak between parallel cases.
- Prefer typed assertions through `TestContext` so failures carry useful expected/actual diffs.
- Use `TestCategory::Unit`, `Integration`, or `EdgeCase` consistently for filtering and reports.
- The custom runner is parallel by default; a case must not mutate shared repository files.
- Keep the Rust plugin-structure port and the Nushell checker aligned while both are supported.
