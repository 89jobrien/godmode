# Metric collection and interpretation

Read this before collecting health metrics 1 to 3 (test count, clippy warnings, TODO density), or before interpreting them for a scorecard. Metrics 4 to 7 are in metrics-collection-api-and-dependencies.md and metrics-collection-module-size-and-docs.md.

## Seven Metrics

### 1. Test Count

**Collection**: Run `cargo nextest list --workspace` (compiles but does not
execute tests) and count output lines containing `::`. Each line is one test.
Fallback: `cargo test --no-run` and count binary artifacts.

**Interpretation**: More tests = better coverage. Trend: increases indicate more
comprehensive testing. Decreases may indicate cleanup or removed features.

**Example**: "272 tests" or "Ran 272 tests"

### 2. Clippy Warnings

**Collection**: Run `taskit check lint` (compile-only, no test
execution) and count warning lines. Use `--message-format json` and count
lines containing `"level":"warning"`. Count only warnings, not notes or errors.

<!-- TODO: bare `grep` above is an anti-pattern for agent use. When collecting in an
     agent context, use the Grep tool instead of piping through grep | wc -l. -->

**Interpretation**: Zero is ideal. Each warning represents a potential code quality
issue. Trend: increases indicate code quality degradation; decreases indicate
refactoring or fixes.

**Example**: `0` or `5`

### 3. TODO/FIXME Density

**Collection**: Use Grep to search for `TODO|FIXME` pattern across all Rust files
in `crates/`. Divide match count by total Rust line count × 1000 to get density
per thousand lines.

**Interpretation**: Lower is better. High density indicates unresolved technical
decisions or incomplete implementations. Expressed as "N per 1,000 lines of code".

**Example**: "0.5 per 1k lines" means roughly one TODO per 2,000 lines.
