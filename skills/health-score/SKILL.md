---
name: "godmode:health-score"
description: >
  Measure codebase health across seven metrics: test count, clippy warnings, TODO
  density, public API surface, dependency count, average module size, and doc
  coverage. Track trends vs previous runs and persist scores to history.
---

# Health Score

Measure codebase health metrics on demand. Produces a scorecard with seven metrics
and tracks trends over time by persisting scores to a JSON history file.

## When to Use

- When asked to run a "health check" or "codebase score"
- During retrospectives or after significant refactoring work
- When evaluating whether technical debt is accumulating
- To baseline a project before proposing major changes
- After a release or quarterly checkpoint

## Seven Metrics

### 1. Test Count

**Collection**: Run `cargo nextest list --workspace` (compiles but does not
execute tests) and count output lines containing `::`. Each line is one test.
Fallback: `cargo test --no-run` and count binary artifacts.

**Interpretation**: More tests = better coverage. Trend: increases indicate more
comprehensive testing. Decreases may indicate cleanup or removed features.

**Example**: "272 tests" or "Ran 272 tests"

### 2. Clippy Warnings

**Collection**: Run `cargo clippy --workspace` (compile-only, no test
execution) and count warning lines. Use `--message-format json` and count
lines containing `"level":"warning"`, or pipe stderr through
`grep "warning\[" | wc -l`. Count only warnings, not notes or errors.

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

## Trend Calculation

For each metric, compare current value to the most recent previous value:

### Improvement threshold (5% change)

- **test_count**: 5% increase is "better"
- **clippy_warnings**: 5% decrease is "better"
- **todo_density**: 5% decrease is "better"
- **api_surface**: 5% change (either direction) noted; growth is neutral, shrink is slight improvement
- **dependency_count**: 5% decrease is "better"
- **avg_module_lines**: 5% decrease is "better"
- **doc_coverage**: 5% increase is "better"

### Trend symbols

- ↑ = **better** (improved by >5%)
- → = **stable** (changed <5%)
- ↓ = **worse** (degraded by >5%)

## History Format

File: `.ctx/memory-bank/health-history.jsonl`

Each line is a valid JSON object with timestamp and all seven metrics:

```json
{
  "timestamp": "2026-06-02T14:35:22Z",
  "test_count": 272,
  "clippy_warnings": 0,
  "todo_density": 0.5,
  "api_surface": 145,
  "dependency_count": 98,
  "avg_module_lines": 180,
  "doc_coverage": 92.5
}
```

- One measurement per line
- Timestamp is ISO 8601 UTC
- All numeric fields are floats or integers as appropriate
- New measurements are appended to the end (immutable log)
- File is created automatically on first write if missing

## Scorecard Format

Markdown table showing current metrics, previous (if available), and trend:

```markdown
## Codebase Health Scorecard

Generated: 2026-06-02 14:35:22 UTC
Workspace: godmode

| Metric          | Current | Previous | Trend |
| --------------- | ------- | -------- | ----- |
| Tests           | 272     | 270      | ↑     |
| Clippy warnings | 0       | 0        | →     |
| TODO density    | 0.5/1k  | 0.6/1k   | ↑     |
| Public API      | 145     | 142      | →     |
| Dependencies    | 98      | 100      | ↑     |
| Avg module      | 180     | 182      | ↑     |
| Doc coverage    | 92.5%   | 91.0%    | ↑     |

### Summary

- **Improving**: 5 metrics
- **Stable**: 2 metrics
- **Degraded**: 0 metrics

### Health Grade: A

Criteria:

- A: test_count ≥ 200, clippy_warnings == 0, doc_coverage > 80%
- B: test_count ≥ 100, no severe clippy issues
- C: otherwise
- Subtract one grade if doc_coverage < 50% or todo_density > 2/1k
```

## Limitations

- **Approximate counts**: Line counts and pattern matches are heuristic-based, not
  exact AST analysis.
- **Not a replacement for coverage tools**: Use `cargo-tarpaulin` or `cargo-llvm-cov`
  for precise code coverage metrics.
- **Clippy is not exhaustive**: Only counts printed warnings; does not assess overall
  code quality.
- **API surface ≠ API complexity**: Counts items, not API design quality or backward
  compatibility.
- **TODO density is contextual**: Some repos legitimately have more TODOs than others
  depending on development stage.
- **Module size varies by domain**: 180 lines average may be appropriate for one crate
  but high for another.

Use these metrics for **trend detection** and **relative comparison**, not absolute
quality judgment.

## See Also

- `health-score-agent.md` — invokes this skill to measure and report
- `CLAUDE.md` — build and test conventions for collecting accurate counts
