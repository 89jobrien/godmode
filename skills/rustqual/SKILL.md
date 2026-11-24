---
name: "rustqual"
description: >
  Rust code quality analysis and improvement using the rustqual CLI. Use when
  the user asks to "run rustqual", "check quality", "fix rustqual findings",
  "improve quality score", "what does rustqual say", or when you see rustqual
  output pasted into the conversation. Also use when configuring rustqual.toml,
  adding qual:allow suppressions, interpreting IOSP/DRY/SRP/TQ findings, or
  working toward a quality score target. Triggers on any mention of rustqual,
  quality score percentages, or finding codes like TQ_UNTESTED, VIOLATION,
  DEAD_CODE, SRP_STRUCT, SRP_MODULE, BOILERPLATE, or TQ_NO_SUT.
color: orange
model: inherit
---

# rustqual

rustqual is a six-dimension Rust code quality analyzer. It scores codebases
on IOSP, Complexity, DRY, SRP, Coupling, Test Quality, and Architecture,
producing a weighted quality score from 0-100%.

## Quick Reference

```bash
rustqual <path>                    # Analyze a crate or workspace
rustqual <path> --no-fail          # Don't exit 1 on findings
rustqual <path> --suggestions      # Show refactoring hints for IOSP
rustqual <path> --json             # Machine-readable output
rustqual <path> --diff             # Only analyze changed files vs HEAD
rustqual --init                    # Generate default rustqual.toml
```

## The Six Dimensions

| Dimension    | What it measures                                      | Weight |
| ------------ | ----------------------------------------------------- | ------ |
| IOSP         | Integration/Operation separation — no mixing          | 0.22   |
| Complexity   | Cognitive/cyclomatic complexity, nesting, length      | 0.18   |
| DRY          | Duplicate code, dead code, boilerplate                | 0.13   |
| SRP          | Single responsibility — cohesion, module size, params | 0.18   |
| Coupling     | Fan-in/out, instability, circular deps                | 0.09   |
| Test Quality | Coverage, naming, untested functions                  | 0.10   |
| Architecture | Layer rule compliance                                 | 0.10   |

## Finding Codes

Each finding has a code that tells you what category it belongs to and
how to fix it:

### IOSP Findings

- **VIOLATION** (logic + calls): A function mixes decision logic (`if`, `match`,
  loops) with calls to other functions. IOSP says a function should either
  contain logic OR delegate to other functions, not both.

  **Fix pattern**: Extract the logic into a pure function, or extract the I/O
  into a helper, so the parent becomes a pure orchestrator (calls only) or
  pure logic (no calls).

  **When to suppress**: I/O boundary functions (file existence check + read)
  and integration roots (`main`, CLI dispatch) inherently mix logic with calls.
  Use `// qual:allow(iosp) reason: "..."` for these.

### Test Quality Findings

- **TQ_UNTESTED**: No unit test calls this function directly. rustqual only
  sees unit tests within the same crate — integration tests in `tests/` and
  cross-crate usage are invisible to it.

  **Fix pattern**: Move testable logic from binary crates to the library crate
  where it can be unit tested. For CLI dispatch functions that are pure I/O
  wiring, suppression via `ignore_functions` in config is appropriate.

- **TQ_NO_SUT**: Test name doesn't reference a recognizable system-under-test.
  rustqual expects test names to include the function or type being tested.

  **Fix pattern**: Rename the test to include the function name. Call the
  function directly (e.g., `WikiCategory::from_str(...)` instead of
  `"...".parse::<WikiCategory>()`), since rustqual traces direct calls.

### DRY Findings

- **DEAD_CODE** (testonly): A function is only called from test code within
  the same crate. This is a frequent false positive for pub API methods used
  by other crates in the workspace.

  **Important**: DEAD_CODE cannot be suppressed with `// qual:allow(dry)`.
  rustqual will flag the allow comment as an ORPHAN_SUPPRESSION. Instead,
  disable dead code detection in `rustqual.toml` if all findings are
  cross-crate false positives:

  ```toml
  [duplicates]
  detect_dead_code = false
  ```

- **BOILERPLATE** (BP-002): Manual trait impl that could use a derive macro.
  Common for `Display`, `Error`, `From` impls.

  **Fix pattern**: Use `thiserror::Error` derive instead of manual
  `impl Display + impl Error`.

### SRP Findings

- **SRP_STRUCT** (LCOM4=N): Struct has low cohesion — its methods form N
  independent clusters. Methods in different clusters don't share fields.

  **Fix pattern**: Split the struct into separate types, or extract a group
  of methods into a standalone function that takes the struct as a parameter.

- **SRP_MODULE** (N lines): File exceeds the configured length ceiling.

  **Fix pattern**: Split into submodules. For test files, use `mod init {}`,
  `mod graph {}`, etc.

## Inline Suppressions

Suppress specific findings with comments above the function:

```rust
// qual:allow(iosp) reason: "I/O boundary function"
fn read_data(path: &Path) -> Result<Data> { ... }
```

Available dimensions: `iosp`, `complexity`, `dry`, `srp`, `coupling`,
`test`, `architecture`.

A bare `// qual:allow` (no dimension) suppresses all dimensions.

**Gotchas**:

- `qual:allow(dry)` does NOT suppress DEAD_CODE findings — only duplicate
  and boilerplate findings. DEAD_CODE is reported under DRY but is not
  suppressible inline. Use `detect_dead_code = false` in config instead.
- Stale or mismatched suppressions produce ORPHAN_SUPPRESSION findings.
- The `max_suppression_ratio` config (default 0.05) warns if too many
  functions are suppressed.

## Configuration (rustqual.toml)

Generate a tailored config with `rustqual --init`, then adjust it. The file
covers ignore lists, IOSP tuning flags, and the complexity, duplicates, SRP,
and weights tables.

Read `references/rustqual-toml-example.md` before writing or editing
`rustqual.toml`. It has the full annotated example.

## Workflow: Improving a Quality Score

When asked to fix rustqual findings, follow this priority order. Each
category has different cost/benefit — start where the impact is highest.

### 1. Low-hanging fruit (high impact, easy fixes)

- **BOILERPLATE**: Switch to derive macros (`thiserror`, `derive_more`)
- **TQ_NO_SUT**: Rename tests to include the SUT function name
- **SRP_MODULE**: Split large files into submodules

### 2. Structural improvements (medium effort)

- **IOSP VIOLATION**: Extract pure logic from I/O functions. Common pattern:
  split `open()` into `read_data()` (I/O) + `from_data()` (logic).
- **TQ_UNTESTED**: Move testable logic from binary crates to library crates.
  Add unit tests for the moved functions.
- **SRP_STRUCT**: Extract method clusters into free functions or new types.
- **High parameter count**: Introduce input structs (like `EdgeInput`).

### 3. Configuration (for false positives)

- **TQ_UNTESTED on CLI dispatch**: Add dispatch function names to
  `ignore_functions` or accept — integration tests cover them.
- **DEAD_CODE on pub API**: Set `detect_dead_code = false` if all findings
  are cross-crate consumers.
- **IOSP on I/O boundaries**: Use `// qual:allow(iosp)` with a reason.

### 4. Know when to stop

Some findings are structural limitations of single-crate analysis:

- CLI dispatch functions will always be TQ_UNTESTED (integration tests
  are invisible)
- Cross-crate pub API will always be DEAD_CODE (no cross-crate tracing)
- I/O boundaries inherently mix logic with calls

Configure or suppress these rather than over-refactoring to satisfy the tool.

## Common Pitfalls

- **Splitting functions to fix IOSP can increase TQ_UNTESTED**: If you
  extract 5 sub-functions from one function, you now have 5 untested
  functions instead of 1. Only split when the resulting functions are
  independently testable.

- **Don't chase 100% by disabling checks**: Use `detect_dead_code = false`
  and `ignore_functions` only for documented false positives. Document
  the reasoning in config comments.

- **Read transcripts, not just scores**: A score increase from refactoring
  that makes code harder to read is not an improvement. The score serves
  the code, not the other way around.

## Baselines and CI Integration

Save a baseline for CI regression detection:

```bash
rustqual --init                                      # Generate tailored config
rustqual --save-baseline .ctx/rustqual-baseline.json # Snapshot current state
rustqual --compare .ctx/rustqual-baseline.json --fail-on-regression  # CI gate
```

This exits 0 as long as the score doesn't drop, even if findings exist.
Commit both `rustqual.toml` and `.ctx/rustqual-baseline.json` to the repo.

## Workspace vs Per-Crate Scanning

rustqual produces different totals depending on scan scope:

- **Workspace-level** (`rustqual .` from repo root): scans all `.rs` files
  including top-level re-exports and files not inside any single crate.
- **Per-crate** (`rustqual crates/<name>`): scans only that crate's `src/`.

The per-crate sum will be lower than the workspace total because the
workspace scan includes files outside any crate boundary. These numbers
are not comparable — always use the same method when tracking progress.

**Rule:** Use `--save-baseline` / `--compare` instead of recording numbers
in plan documents. The baseline file is the single source of truth.

## Autonomous Quality Loop

When asked to "improve quality" or "run the quality loop", read
`references/autonomous-quality-loop.md` first. It defines the MEASURE and FIX
phases with their commands, scan scope, and exit criteria. The VERIFY and COMMIT
phases are in `references/autonomous-quality-loop-verify-and-commit.md`. Run all
four phases in order and do not skip MEASURE, because every
later comparison depends on the baseline it saves.

## cfg-gated Code (Kani, test utilities)

`#[cfg(kani)]` modules and `#[cfg(any(test, feature = "testutil"))]` code
are invisible to rustqual's dead-code analysis. Read
`references/cfg-gated-code.md` before choosing between moving that code to
its own file and listing it in `exclude_files`, or turning off dead-code
detection.

## Reference Files

- **`references/cli-reference.md`** — Full CLI options, output formats,
  config file schema, exit codes, CI examples
- **`references/fix-patterns.md`** — Concrete before/after Rust code
  examples for every finding type
- **`references/workflow.md`** — Step-by-step walkthrough of a typical
  improvement session from first run to 100%, with progression table
- **`references/rustqual-toml-example.md`** — Annotated `rustqual.toml`
  example. Read before writing or editing the config.
- **`references/autonomous-quality-loop.md`** — The MEASURE and FIX phases
  with commands and exit criteria. Read before running "improve quality" or
  "run the quality loop".
- **`references/autonomous-quality-loop-verify-and-commit.md`** — The VERIFY and
  COMMIT phases. Read after FIX, before verifying or committing the baseline.
- **`references/cfg-gated-code.md`** — How to handle `#[cfg(kani)]` and
  cfg-gated test utilities in dead-code findings. Read when a finding
  involves cfg-gated code.
