# Phases 3-4: Hypotheses and instrumentation

Read this before ranking hypotheses about a failure (Phase 3), or before adding debug probes or measuring a performance regression (Phase 4).

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
