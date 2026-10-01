# Bug Reproduction Record

## Symptom

> One sentence: what is observed vs. what is expected.

## Feedback Loop (Phase 0)

> The exact command or script that triggers the bug. One line, copy-paste ready.

```bash

```

- Loop runtime: \_\_\_
- Deterministic? (yes / no)
- Reproduction rate: N of M runs

## Reproduction Steps

```bash
# Exact commands to reproduce
cargo nextest run -p <crate> -- <test_name>
```

## Error Output (verbatim)

```text
<paste full error here — do not summarise>
```

## Recent Changes

```bash
git log --oneline -5
git diff HEAD~1 -- <relevant files>
```

## Ranked Hypotheses (Phase 3)

3–5 rows. Each must state a falsifiable prediction.

| #   | Hypothesis | Prediction if true | Probe | Result |
| --- | ---------- | ------------------ | ----- | ------ |
| 1   |            |                    |       |        |
| 2   |            |                    |       |        |
| 3   |            |                    |       |        |
| 4   |            |                    |       |        |
| 5   |            |                    |       |        |

## Root Cause (confirmed)

> Fill in after Phase 3 confirms the hypothesis.

## Regression Test Seam (Phase 5)

- Seam: \_\_\_
- Exercises the real bug pattern at the call site? (yes / no — if no, that is the finding)

## Fix

> One sentence describing the change.

## Cleanup (Phase 6)

- [ ] Original repro no longer reproduces
- [ ] Regression test passes (or missing seam documented)
- [ ] `rg '\[DEBUG-'` returns nothing
- [ ] Throwaway prototypes deleted
- [ ] Root cause stated in the commit message

## Preventable by?

> Architectural follow-up, or none.
