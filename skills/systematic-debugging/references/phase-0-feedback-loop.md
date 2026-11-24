# Phase 0: Build a feedback loop

Read this before starting any debugging task, and before concluding that a bug cannot be reproduced on demand.

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
