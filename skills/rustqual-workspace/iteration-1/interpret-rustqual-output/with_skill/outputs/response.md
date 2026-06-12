# Improving Your rustqual Score: 62.3% → Target 85%+

## Current State

```
Quality Score: 62.3%    22 findings

IOSP:         78.0%  (3 violations)
Complexity:   90.0%
DRY:          70.0%  (4 dead code, 1 boilerplate)
SRP:          55.0%  (1 struct LCOM4=3, 1 module 450 lines)
Test Quality: 40.0%  (10 untested, 3 no-sut)
Architecture: 100.0%
```

The biggest score drags, weighted by their dimension weight:

| Dimension    | Weight | Score | Impact |
| ------------ | ------ | ----- | ------ |
| SRP          | 0.18   | 55%   | High   |
| IOSP         | 0.22   | 78%   | Medium |
| DRY          | 0.13   | 70%   | Medium |
| Test Quality | 0.10   | 40%   | Low    |

## Context: Workspace-Specific Situation

Two important facts change the remediation plan:

1. **Dead code findings are false positives.** The 4 `DEAD_CODE` findings are pub functions
   in the lib crate consumed by the CLI binary. rustqual only traces within a single crate,
   so cross-crate usage is invisible to it. These are not real dead code.

2. **TQ_UNTESTED is split between crates.** 4 untested functions are in the lib (genuine
   findings — add tests), and 6 are in the CLI binary (structural false positives — CLI
   dispatch functions are covered by integration tests that rustqual cannot see).

## Best Path: Four Rounds in Priority Order

### Round 1: Quick wins (low effort, ~5-8 point gain)

**1a. Fix TQ_NO_SUT (3 findings)**

Rename the 3 tests whose names don't reference the function under test. rustqual traces
direct function calls, so also make sure each test calls the SUT by name rather than via
a trait method or string parse:

```rust
// Before (flagged):
#[test]
fn test_parsing() { ... }

// After (clear):
#[test]
fn parse_entity_from_str_returns_error_on_empty() {
    let result = Entity::from_str("");
    assert!(result.is_err());
}
```

**1b. Fix BOILERPLATE (1 finding)**

Replace the manual `impl Display + impl Error` with `thiserror::Error`:

```rust
// Before:
impl fmt::Display for MyError { ... }
impl std::error::Error for MyError {}

// After:
#[derive(thiserror::Error, Debug)]
enum MyError {
    #[error("not found: {0}")]
    NotFound(String),
}
```

**1c. Split the 450-line module (SRP_MODULE)**

Split the large file into submodules. For test files use `mod` blocks by feature area;
for source files create separate `src/submodule.rs` files. This eliminates the SRP_MODULE
finding and improves the SRP score meaningfully given its 0.18 weight.

After Round 1, expect roughly 68-72%.

---

### Round 2: Configure away false positives (zero refactoring, ~6-8 point gain)

**2a. Disable dead code detection**

All 4 DEAD_CODE findings are cross-crate pub API — genuine false positives. Disable the
check in `rustqual.toml`:

```toml
[duplicates]
# DEAD_CODE findings are all pub API consumed by the CLI binary crate.
# Cross-crate usage is invisible to single-crate analysis.
detect_dead_code = false
```

Do NOT use `// qual:allow(dry)` for these — that suppression does not cover DEAD_CODE and
will produce ORPHAN_SUPPRESSION findings instead.

**2b. Suppress CLI dispatch functions for TQ_UNTESTED**

The 6 untested functions in the CLI binary are dispatch functions (integration roots). They
are covered by integration tests that rustqual cannot trace. Add them to `ignore_functions`:

```toml
# rustqual.toml
ignore_functions = ["main", "cmd_*", "run_*"]
```

Adjust the glob patterns to match your actual CLI function naming convention.

After Round 2, expect roughly 78-82%.

---

### Round 3: Add unit tests for lib functions (medium effort, ~4-6 point gain)

The 4 untested functions in the lib crate are genuine findings. Add unit tests directly in
the lib crate (in `#[cfg(test)]` modules in each source file, or in a `tests/` integration
test that imports from the lib). Each function that gets a test eliminates one TQ_UNTESTED
finding.

Prioritize testable pure logic first — avoid testing functions that primarily do I/O, since
those should be restructured (see Round 4) or suppressed.

After Round 3, expect roughly 83-87%.

---

### Round 4: Fix IOSP violations (medium effort, remaining gains)

For the 3 IOSP violations, evaluate each one:

- **CLI dispatch / integration roots** (`cmd_*`, `main`): suppress with inline annotation —
  these inherently mix logic with I/O calls and are not worth refactoring:

  ```rust
  // qual:allow(iosp) reason: "CLI integration root"
  fn cmd_ingest(args: IngestArgs) -> Result<()> { ... }
  ```

- **Domain functions that mix logic with calls**: extract the logic into a pure function.
  Common pattern: split `open()` into `read_data()` (I/O only) + `from_data()` (logic only).

**Watch out**: splitting IOSP violations creates new functions that will appear as
TQ_UNTESTED. Only split when the resulting functions are independently testable — otherwise
suppress the IOSP violation and move on.

---

### Round 5: SRP_STRUCT (LCOM4=3)

The struct with LCOM4=3 has three independent method clusters that don't share fields.
Options:

- Extract one cluster into free functions that take the struct as a parameter
- Split the struct into two types with focused responsibilities

This is the highest-effort fix. Defer it until after the other rounds to avoid churn —
it may interact with the IOSP refactoring.

---

## Summary Table

| Round | Action                                                   | Findings resolved | Est. score |
| ----- | -------------------------------------------------------- | ----------------- | ---------- |
| Start | —                                                        | —                 | 62.3%      |
| 1     | TQ_NO_SUT renames, thiserror, split 450-line module      | 4                 | ~70%       |
| 2     | `detect_dead_code = false`, `ignore_functions` for CLI   | 10                | ~80%       |
| 3     | Add tests for 4 lib functions                            | 4                 | ~85%       |
| 4     | IOSP: suppress CLI roots, extract logic where worthwhile | 3                 | ~88%       |
| 5     | SRP_STRUCT: split or extract method cluster              | 1                 | ~90%+      |

## Key Rules for This Workspace

- **Never use `// qual:allow(dry)` for DEAD_CODE** — it produces ORPHAN_SUPPRESSION instead.
  Use `detect_dead_code = false` in config.
- **Don't split IOSP violations blindly** — each new function is another TQ_UNTESTED. Only
  split when you'll also add a test for the extracted function.
- **Rounds 2 and 3 give the most points per hour of effort** — configuration changes are
  zero-risk and resolve 10 findings immediately.
- Run `cargo test && cargo clippy` between rounds. Refactoring can break things.
