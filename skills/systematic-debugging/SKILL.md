---
name: "godmode:systematic-debugging"
description: >
  Use when encountering any bug, test failure, panic, unexpected behavior, flaky test, or
  performance regression — before proposing any fix. Triggers on error output, failing tests,
  "why is X not working", "this is broken", "this is flaky", "this got slower", or any
  symptom description.
requires: []
next: [task-driven-development]
---

# Systematic Debugging

**Rule zero**: No fix without a root cause. Symptom patches that mask underlying problems
are not fixes.

**Rule zero-prime**: No hypothesis without a loop. If you cannot trigger the bug on demand,
you cannot test any theory about it.

Skip phases only when you can state the justification out loud.

## Phase 0 — Build a feedback loop

**This phase is the skill.** Everything after it is mechanical. A fast, deterministic,
agent-runnable pass/fail signal turns bisection, hypothesis testing, and instrumentation into
consumers of one signal. Without it you are only staring at code.

Spend disproportionate effort here. Be aggressive. Be creative. Refuse to give up.

### Ways to construct one — roughly in this order

1. **Failing test** at whatever seam reaches the bug — unit, integration, e2e.
2. **Curl / HTTP script** against a running dev server.
3. **CLI invocation** with a fixture input, diffing stdout against a known-good snapshot.
4. **Headless browser script** (Playwright / Puppeteer) — drives the UI, asserts on
   DOM/console/network.
5. **Replay a captured trace** — save a real request, payload, or event log to disk and replay
   it through the code path in isolation.
6. **Throwaway harness** — minimal subset of the system (one service, mocked deps) that
   exercises the bug path from a single function call.
7. **Property / fuzz loop** — for "sometimes wrong output", run 1000 random inputs and look for
   the failure mode.
8. **Bisection harness** — if the bug appeared between two known states (commit, dataset,
   version), automate "boot at state X, check, repeat" so `git bisect run` works.
9. **Differential loop** — same input through old-version vs new-version (or two configs), then
   diff the outputs.
10. **HITL script** — last resort, when a human must click. Drive the human from a template so
    the loop stays structured and the captured output feeds back in.

### Iterate on the loop itself

Once you have _a_ loop, treat it as a product:

- **Faster?** Cache setup, skip unrelated init, narrow the test scope.
- **Sharper?** Assert on the specific symptom, not "didn't crash".
- **More deterministic?** Pin the clock, seed the RNG, isolate the filesystem, freeze the
  network.

A 30-second flaky loop is barely better than no loop. A 2-second deterministic loop is a
debugging superpower.

### Non-deterministic bugs

The goal is not a clean repro, it is a **higher reproduction rate**. Loop the trigger 100x,
parallelise it, add stress, narrow the timing window, inject sleeps. A 50%-flake bug is
debuggable; 1% is not. Keep raising the rate until it is debuggable.

### When you genuinely cannot build a loop

Stop and say so explicitly. List what you tried, then ask the user for:

- (a) access to whatever environment reproduces it,
- (b) a captured artifact (HAR file, log dump, core dump, timestamped screen recording), or
- (c) permission to add temporary production instrumentation.

Do not proceed to Phase 1 without a loop you believe in.

## Phase 1 — Reproduce

Run the loop. Watch the bug appear. Then confirm:

- The loop produces the failure mode the **user** described, not a different failure that
  happens to be nearby. Wrong bug = wrong fix.
- It reproduces across multiple runs, or at a high enough rate to debug against.
- You captured the exact symptom (error message, wrong output, slow timing) so later phases
  can verify the fix actually addresses it.

Record it in `helpers/repro-template.md`. For Rust, `helpers/debug-session.nu` automates
the git + nextest capture; pass the crate and an optional test name as arguments.

## Phase 2 — Locate the root cause

1. **Parse the error exactly.** Read the full message, stack trace, and surrounding context.
   Do not skim.
2. **Check environment first.** Missing env vars and unresolved `op://` refs are the most
   common root cause. Verify before investigating code.
3. **Check recent changes.**

   ```bash
   git diff HEAD~1
   git log --oneline -10
   ```

4. **For multi-component failures**: add diagnostic output at each boundary to locate where
   the failure originates. Trace data flow backward from the symptom.
5. **Find a working analog** in the codebase — similar code that does work. Compare working
   vs. broken line by line. Document every difference, however insignificant it looks.

## Phase 3 — Rank and falsify hypotheses

Generate **3–5 ranked hypotheses** before testing any of them. Single-hypothesis generation
anchors on the first plausible idea and throws away the rest.

Each must be falsifiable — state the prediction it makes:

> "If `<cause>` is the cause, then `<observable change>` will make the bug disappear / make
> it worse."

If you cannot state the prediction, the hypothesis is a vibe. Discard it or sharpen it.

**Show the ranked list to the user before testing.** They often have domain knowledge that
re-ranks it instantly ("we just shipped a change to #3") or know what is already ruled out.
Cheap checkpoint, big time saver. Do not block on it — proceed with your ranking if the user is
AFK.

## Phase 4 — Instrument

Every probe must map to a specific prediction from Phase 3. **One variable at a time.**

1. **Debugger / REPL inspection** if the environment supports it. One breakpoint beats ten
   logs.
2. **Targeted logs** at the boundaries that distinguish the hypotheses.
3. Never "log everything and grep".

**Tag every debug log with a unique prefix**, e.g. `[DEBUG-a4f2]`. Cleanup at the end becomes
a single `rg '\[DEBUG-'`. Untagged logs survive; tagged logs die.

**Perf branch.** For performance regressions, logs are the wrong tool. Establish a baseline
measurement first (timing harness, `std::time::Instant`, profiler, query plan), then bisect.
Measure first, fix second.

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

## 3-Failure Rule

If 3 sequential fix attempts all fail, stop. The architecture likely has a deeper problem.
Surface it to the user rather than continuing to patch.

## Rust-Specific Checklist

- `RUST_BACKTRACE=1` or `RUST_BACKTRACE=full` for panics
- `RUST_LOG=debug` for tracing output
- `cargo check` before `cargo nextest run` — catch compile errors cheaply
- `cargo clippy -p <crate> -- -D warnings` — warnings often point at the root cause
- Lifetime and borrow errors: read the full compiler message, not just the first line
- Async failures: check executor context (tokio runtime not entered, etc.)
- Cross-crate failures: check feature flags and conditional compilation gates
- Flaky tests: `cargo nextest run --retries 0` surfaces hidden order dependencies; then
  `--test-threads 1` to confirm it is parallelism, not logic

## Never

- Apply a fix before identifying root cause
- Form a hypothesis before you have a loop that triggers the bug
- Log without a `[DEBUG-` tag
- Stack multiple fixes in one commit
- Suppress compiler warnings to make a test pass
- Use `unwrap()` to "fix" an error — propagate it properly

## Additional Resources

- **`references/rust-debug-checklist.md`** — environment checks, backtrace commands,
  async/lifetime failure patterns
- **`references/feedback-loops.md`** — concrete loop recipes per bug class, plus the perf and
  bisection harnesses
- **`helpers/repro-template.md`** — bug reproduction record to fill in during Phase 1
- **`helpers/debug-session.nu`** — scripted Phase 1 capture (git diff + `cargo nextest run`)
