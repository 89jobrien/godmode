# Metric collection and interpretation

Read this before collecting any of the seven health metrics, or before interpreting a metric's value for a scorecard.

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

### 4. Public API Surface

**Collection**: Count all `pub fn`, `pub struct`, `pub enum`, `pub trait`
declarations in `crates/*/src/**/*.rs` using Grep with pattern
`^\s*pub\s+(fn|struct|enum|trait)`. Sum across workspace.

**Interpretation**: Growing API surface can indicate feature expansion or API
bloat. Stable or shrinking surface may indicate consolidation. Use as a trend
indicator, not an absolute measure.

**Example**: "145 public items"

### 5. Dependency Count

**Collection**: Read `Cargo.lock` and count `[[package]]` section markers. This
includes all direct and transitive dependencies.

**Interpretation**: More dependencies = larger attack surface and more maintenance
burden. Fewer dependencies = simpler dependency tree. Trend is most useful:
increases may signal feature additions; decreases indicate cleanup.

**Example**: "98 total dependencies"

### 6. Average Module Line Count

**Collection**: List all `.rs` files in `crates/*/src/`. Count lines in each
with `wc -l`. Compute arithmetic mean. Round to nearest integer.

**Interpretation**: Higher average module size may indicate modules are doing too
much. Lower average indicates more focused modules. Ideal range is 100–300 lines
per module depending on domain.

**Example**: "180 lines per module"

### 7. Documentation Coverage

**Collection**: Count all items with `pub` visibility (functions, structs, enums,
traits). Count how many are immediately preceded by `///` doc comments. Compute
percentage: (documented / total) × 100. Round to one decimal place.

**Interpretation**: Higher is better. 80%+ is good; <50% indicates undocumented
public API. Public items without docs are harder for consumers to use correctly.

**Example**: "92.5% (138 of 149 public items documented)"
