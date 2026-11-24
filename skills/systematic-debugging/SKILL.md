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

## Phases

Read the file named in a row before doing the work in that phase. These files are not loaded
automatically.

| File                                            | Read before                                                                                         |
| ----------------------------------------------- | --------------------------------------------------------------------------------------------------- |
| `references/phase-0-feedback-loop.md`           | starting any debugging task, or concluding a bug cannot be reproduced on demand                     |
| `references/phase-1-2-reproduce-locate.md`      | confirming a bug reproduces (Phase 1), or tracing it to a root cause (Phase 2)                      |
| `references/phase-3-4-hypotheses-instrument.md` | ranking hypotheses (Phase 3), or adding probes or measuring a performance regression (Phase 4)      |
| `references/phase-5-6-fix-cleanup.md`           | writing the regression test and fix (Phase 5), or closing out a debugging task (Phase 6)            |
| `references/rust-specific-checklist.md`         | debugging a panic, borrow or lifetime error, async failure, cross-crate failure, or flaky Rust test |

## 3-Failure Rule

If 3 sequential fix attempts all fail, stop. The architecture likely has a deeper problem.
Surface it to the user rather than continuing to patch.

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
- **`helpers/debug-session.nu`** — scripted Phase 1 capture (git diff + `taskit test run`)
