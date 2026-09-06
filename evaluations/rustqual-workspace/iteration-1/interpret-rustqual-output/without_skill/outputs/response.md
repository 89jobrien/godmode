# Interpreting rustqual output and improving score

## Score breakdown analysis

Current: 62.3% with 22 findings. The two biggest drags are Test Quality (40%) and SRP (55%).
Fixing those two categories alone would move the needle more than fixing everything else combined.

## Dead code false positives (DRY: 70%)

The 4 dead code findings on `pub` functions in the lib crate are almost certainly false positives.
rustqual analyzes crates in isolation by default — it sees `pub fn foo()` with no callers inside
the lib crate and flags it. The CLI binary consuming those functions is a separate crate.

Fix: Add `#[allow(dead_code)]` is the wrong approach. Instead:

1. Check if rustqual has a `--workspace` or `--all` flag that analyzes the full dependency graph
   across crates. If so, run it at workspace level rather than per-crate.
2. If rustqual doesn't support cross-crate analysis, suppress the findings via config
   (e.g., `rustqual.toml` exclusion for `pub` items in lib crates) rather than adding
   `#[allow]` noise to every function.
3. The boilerplate finding (1) is worth investigating — if it's a `Display` impl or similar
   derived trait scaffolding, it may also be suppressible as acceptable boilerplate.

Expected gain: recover most of the DRY penalty (up to ~+6 points on the overall score).

## SRP violations (55%) — second priority

Two findings: one struct with LCOM4=3, one module at 450 lines.

- **LCOM4=3 struct**: LCOM4 measures lack of cohesion — a score of 3 means the struct has 3
  independent clusters of methods that don't share fields. Split it into 2-3 smaller structs,
  each owning one cluster. This is usually a 30-60 minute refactor.
- **450-line module**: Extract cohesive subsets into submodules. A module that large typically
  has 2-3 logical groupings. Splitting into `mod foo { ... }` inline or separate files both
  satisfy the metric.

Expected gain: SRP at 55% is the second-largest drag. Resolving both findings should push it
to 80-90%, adding roughly +4-6 points to overall score.

## Test Quality (40%) — highest impact, most effort

10 untested functions split 4 (lib) + 6 (CLI binary). 3 functions have no SUT reference.

Priority order:

1. **Lib crate (4 untested)**: Write unit tests directly. Lib functions are easiest to test in
   isolation. Focus here first — higher confidence per test written.
2. **CLI binary (6 untested)**: CLI functions are harder to unit test. Two approaches:
   - Extract business logic out of command handlers into lib functions, then test the lib.
     This also improves IOSP (separation of orchestration from logic).
   - Use `assert_cmd` or `trycmd` for integration tests that invoke the binary — rustqual
     typically counts these if the SUT is referenced.
3. **3 no-SUT findings**: These are functions where tests exist but don't reference the
   function under test by name (e.g., testing indirectly through a wrapper). Add direct calls
   to the function in the test body.

Expected gain: Test Quality at 40% with 10 untested is the single largest score driver.
Getting to 70% test coverage would add roughly +12 points to overall score.

## IOSP violations (78%) — lower priority

3 violations at 78% — already decent. IOSP flags functions that mix orchestration (calling
other functions) with logic (conditionals, computation). Common pattern in CLI binaries where
command handlers do both dispatch and logic inline.

Fix: Push logic into pure functions, keep handlers as thin orchestrators. This also helps
testability, so fixing IOSP often reduces the untested count simultaneously.

Expected gain: minor (+1-2 points) but synergizes with test quality improvements.

## Recommended sequence

1. **Fix dead code cross-crate analysis** (30 min) — check workspace-level rustqual invocation.
   Eliminates false positives without code changes.
2. **Split the LCOM4=3 struct** (1-2 hours) — most contained SRP fix.
3. **Write lib crate tests** (2-3 hours) — 4 functions, highest ROI per test.
4. **Extract CLI logic to lib, add tests** (3-4 hours) — addresses both IOSP and test quality.
5. **Split 450-line module** (1 hour) — mechanical extraction.
6. **Fix no-SUT test references** (30 min) — add direct function calls to existing tests.

Realistic score after completing all of the above: **82-88%**, driven primarily by Test Quality
recovering from 40% to ~75-80%.
