# Finding codes: IOSP and test quality

Read this before fixing or suppressing an IOSP VIOLATION, TQ_UNTESTED, or TQ_NO_SUT finding.

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
