# Improvement workflow

Read this when asked to fix rustqual findings or raise the quality score, to get the priority order.

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
