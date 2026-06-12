# rustqual Workflow: From First Run to 100%

A step-by-step walkthrough of a typical rustqual improvement session,
based on taking a Rust workspace from 44% to 100%.

## Phase 1: Baseline

```bash
rustqual <path>
```

Read the summary. Note which dimensions are dragging the score down.
A typical first run might look like:

```
Quality Score: 44.2%    38 findings

IOSP:         72.0%  (4 violations)
Complexity:   85.0%
DRY:          80.0%  (3 dead code, 2 boilerplate)
SRP:          60.0%  (2 struct, 1 module)
Test Quality: 55.0%  (15 untested)
```

Generate a config file to lock in current thresholds:

```bash
rustqual --init
```

## Phase 2: Triage findings

Group findings by actionability:

**Fix now** (high impact, low effort):

- BOILERPLATE — switch to derive macros
- TQ_NO_SUT — rename tests
- SRP_MODULE — split files into modules

**Fix with refactoring** (medium effort):

- IOSP VIOLATION — extract logic from I/O
- TQ_UNTESTED — move logic from bin to lib crate
- SRP_STRUCT — reduce LCOM4 by extracting methods
- High parameter count — introduce input structs

**Configure away** (false positives):

- TQ_UNTESTED on CLI dispatch — `ignore_functions`
- DEAD_CODE on cross-crate pub API — `detect_dead_code = false`
- IOSP on I/O boundaries — `// qual:allow(iosp)`

## Phase 3: Fix in priority order

### Round 1: Low-hanging fruit

1. Replace manual `impl Display + Error` with `thiserror::Error` derive
2. Rename tests to include the SUT function name
3. Split large test files into `mod` blocks by feature area

Run rustqual again. Expect 5-10% improvement.

### Round 2: Move logic to library crate

The biggest TQ_UNTESTED wins come from moving testable logic out of
binary crates:

1. **Parsing functions** — implement `FromStr` on types in the lib
   crate instead of ad-hoc `parse_*` in the binary. Add unit tests.

2. **Business logic** — functions like `ingest_entities()` that
   operate on domain types belong in the lib crate, not the CLI.
   Create a module (e.g., `src/ingest.rs`), move the logic, add
   tests, and have the CLI call the lib version.

3. **Path/layout helpers** — functions like `graph_path(root)` that
   compute file paths are pure logic. Move to lib and test.

Run rustqual again. Each moved function that gets a test eliminates
one TQ_UNTESTED finding.

### Round 3: IOSP violations

For each VIOLATION (logic + calls):

1. Identify the logic (if/match/loops) and the calls (I/O, delegation)
2. Extract one or the other into a separate function
3. The parent becomes either pure orchestration or pure logic

Common patterns:

- `open()` → split into `read_data()` (I/O) + `from_data()` (logic)
- `lint()` → split into `iter_pages()` (I/O) + `build_report()` (logic)
- `cmd_foo()` → these are integration roots, suppress with
  `// qual:allow(iosp)`

**Watch out**: Splitting one function into 5 sub-functions can create
5 new TQ_UNTESTED findings. Only split when the pieces are independently
testable or when you'll suppress via `ignore_functions`.

### Round 4: SRP

- **SRP_STRUCT** (LCOM4=N): Extract a cluster of methods into free
  functions or a new type. If the struct genuinely has two concerns,
  split it.

- **SRP_MODULE**: Already handled in Round 1 (split files).

- **High parameter count**: Introduce input structs. Group related
  parameters into a borrowed struct (use lifetimes).

## Phase 4: Configure for false positives

After all genuine fixes, configure away structural false positives:

```toml
# rustqual.toml

# CLI dispatch functions are covered by integration tests
ignore_functions = ["main", "test_*", "cmd_*"]

[duplicates]
# All DEAD_CODE findings are cross-crate pub API
detect_dead_code = false
```

Add inline suppressions for I/O boundaries:

```rust
// qual:allow(iosp) reason: "I/O boundary — existence check + read"
fn read_graph_data(path: &Path) -> Result<GraphData> { ... }
```

## Phase 5: Verify and lock

```bash
# Final check
rustqual <path>

# Save baseline for CI
rustqual <path> --save-baseline baseline.json
```

## Example progression

| Round | Score | Key actions                                               |
| ----- | ----- | --------------------------------------------------------- |
| Start | 44.2% | 38 findings across all dimensions                         |
| 1     | 58.8% | thiserror, test renames, file splits                      |
| 2     | 70.6% | Extract BFS helper, EdgeInput struct, lint helpers        |
| 3     | 76.9% | FromStr for WikiCategory, path helpers to lib             |
| 4     | 84.7% | ingest logic to lib, split CLI tests into modules         |
| 5     | 90.6% | qual:allow(iosp) on I/O boundaries                        |
| 6     | 100%  | detect_dead_code=false, ignore_functions for CLI dispatch |

## Anti-patterns

- **Don't fix findings one at a time** — group by category and fix
  all of one type before re-running. This avoids churn.

- **Don't split functions just to satisfy IOSP** — if the split
  makes code harder to read or creates more untested functions,
  suppress instead.

- **Don't disable checks to chase 100%** — only disable for
  documented, genuine false positives. A clean 90% is better
  than a hollow 100%.

- **Always run tests between rounds** — refactoring can break things.
  `cargo test && cargo clippy` after every batch of changes.

- **Don't forget `cargo clippy`** — rustqual doesn't check for clippy
  warnings. Run both.
