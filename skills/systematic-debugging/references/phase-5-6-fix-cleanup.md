# Phases 5-6: Fix at the correct seam, then clean up

Read this before writing a regression test or fix for a confirmed root cause (Phase 5), or before declaring a debugging task done (Phase 6).

## Phase 5 — Fix at the correct seam

Write the regression test **before** the fix — but only if there is a **correct seam** for it.

A correct seam exercises the real bug pattern as it occurs at the call site. A single-caller
unit test when the bug needs multiple callers, or a unit test that cannot reproduce the chain
that triggered the bug, gives false confidence.

**If no correct seam exists, that is the finding.** Note it — the architecture is preventing
the bug from being locked down. Carry it into Phase 6.

If a correct seam exists:

1. Turn the minimised repro into a failing test at that seam.
2. Watch it fail.
3. Apply the single fix.
4. Watch it pass.
5. Re-run the Phase 0 loop against the original, un-minimised scenario.

```bash
cargo nextest run -p <crate>
cargo clippy -p <crate> -- -D warnings
```

## Phase 6 — Cleanup and post-mortem

Required before declaring done:

- The original repro no longer reproduces (re-run the Phase 0 loop)
- The regression test passes, or the absence of a seam is documented
- All `[DEBUG-...]` instrumentation removed — `rg '\[DEBUG-'` comes back empty
- Throwaway prototypes deleted, or moved to a clearly-marked debug location
- The confirmed root cause is stated in the commit message, so the next debugger learns it

**Then ask: what would have prevented this bug?** If the answer is architectural — no good
test seam, tangled callers, hidden coupling — record it as a concrete follow-up. Make the
recommendation **after** the fix is in; you know more now than when you started.
