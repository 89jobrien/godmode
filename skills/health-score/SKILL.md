---
name: "godmode:health-score"
description: >
  Measure codebase health across seven metrics: test count, clippy warnings, TODO
  density, public API surface, dependency count, average module size, and doc
  coverage. Track trends vs previous runs and persist scores to history.
requires: []
next: [dead-code, dep-audit]
---

# Health Score

Measure codebase health metrics on demand. Produces a scorecard with seven metrics
and tracks trends over time by persisting scores to a JSON history file.

## When to Use

- When asked to run a "health check" or "codebase score"
- During retrospectives or after significant refactoring work
- When evaluating whether technical debt is accumulating
- To baseline a project before proposing major changes
- After a release or quarterly checkpoint

## Reference files

Read the file named in a row before doing the task in its right column. These files are not
loaded automatically.

| File                                                    | Read before                                                                        |
| ------------------------------------------------------- | ---------------------------------------------------------------------------------- |
| `references/metrics-collection.md`                      | collecting metrics 1 to 3 (tests, clippy, TODOs), or interpreting a metric's value |
| `references/metrics-collection-api-and-dependencies.md` | collecting metrics 4 and 5 (public API surface, dependency count)                  |
| `references/metrics-collection-module-size-and-docs.md` | collecting metrics 6 and 7 (module line count, documentation coverage)             |
| `references/trend-calculation.md`                       | comparing a current measurement to the previous one, or assigning trend symbols    |
| `references/history-and-scorecard.md`                   | appending to the history file                                                      |
| `references/scorecard-format-and-grade.md`              | writing a scorecard for the user, including the health grade criteria              |
| `references/limitations.md`                             | presenting a score as a quality judgment, or comparing scores across repos         |

## See Also

- `health-score-agent.md` — invokes this skill to measure and report
- `CLAUDE.md` — build and test conventions for collecting accurate counts
