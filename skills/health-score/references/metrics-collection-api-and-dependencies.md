# Metric collection: public API surface and dependency count

Read this before collecting the public API surface or dependency count metrics, or before reading their trend for a scorecard.

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
