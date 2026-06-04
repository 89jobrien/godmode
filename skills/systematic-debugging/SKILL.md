---
name: "godmode:systematic-debugging"
description: >
  Use when encountering any bug, test failure, panic, or unexpected behavior — before
  proposing any fix. Triggers on error output, failing tests, "why is X not working",
  "this is broken", or any symptom description.
requires: []
next: [task-driven-development]
---

# Systematic Debugging

**Rule zero**: No fix without a root cause. Symptom patches that mask underlying problems
are not fixes.

## Phase 1: Root Cause Investigation

Before touching any code:

1. **Parse the error exactly.** Read the full message, stack trace, and surrounding context.
   Do not skim.

2. **Reproduce it.** Run the failing test or command and confirm you can reproduce it
   consistently:

   ```bash
   RUST_BACKTRACE=1 cargo nextest run -p <crate> -- <test_name>
   ```

3. **Check recent changes.** What changed last?

   ```bash
   git diff HEAD~1
   git log --oneline -10
   ```

4. **For multi-component failures**: add diagnostic output at each boundary to locate
   where the failure originates. Trace data flow backward from the symptom.

5. **Check environment first.** Missing env vars and unresolved `op://` refs are the most
   common root cause. Verify before investigating code.

## Phase 2: Pattern Analysis

- Find a working analog in the codebase — similar code that does work.
- Compare working vs. broken line by line.
- Document every difference, no matter how insignificant it looks.

## Phase 3: Hypothesis and Testing

- State one specific hypothesis: "The bug is X because Y."
- Test it with a minimal, isolated change.
- If wrong, discard the hypothesis entirely and form a new one.
- Do not layer fixes on top of failed hypotheses.

## Phase 4: Fix

Only after root cause is confirmed:

1. Write a failing test that captures the bug (if one doesn't exist).
2. Implement the single fix.
3. Verify:
   ```bash
   cargo nextest run -p <crate>
   cargo clippy -p <crate> -- -D warnings
   ```

## 3-Failure Rule

If 3 sequential fix attempts all fail, stop. The architecture likely has a deeper problem.
Surface it to the user rather than continuing to patch.

## Rust-Specific Checklist

- `RUST_BACKTRACE=1` or `RUST_BACKTRACE=full` for panics
- `RUST_LOG=debug` for tracing output
- `cargo check` before `cargo nextest` — catch compile errors cheaply
- `cargo clippy -p <crate> -- -D warnings` — warnings often point at the root cause
- Lifetime and borrow errors: read the full compiler message, not just the first line
- Async failures: check executor context (tokio runtime not entered, etc.)
- Cross-crate failures: check feature flags and conditional compilation gates

## Never

- Apply a fix before identifying root cause
- Stack multiple fixes in one commit
- Suppress compiler warnings to make a test pass
- Use `unwrap()` to "fix" an error — propagate it properly

## Additional Resources

- **`references/rust-debug-checklist.md`** — environment checks, backtrace commands, async/lifetime failure patterns
- **`helpers/repro-template.md`** — bug reproduction record to fill in during Phase 1
