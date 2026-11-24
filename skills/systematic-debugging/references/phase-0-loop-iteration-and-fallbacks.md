# Phase 0: iterate on the loop and handle non-deterministic bugs

Read this before tuning a feedback loop, reproducing a non-deterministic bug, or when no loop can be built.

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
