# Metric collection: module size and documentation coverage

Read this before collecting average module line count or documentation coverage, or before reading either for a scorecard.

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
