---
name: "godmode:decompose"
description: >
  Use when the user wants to break up a large diff, PR, or branch into smaller independent PRs
  or branches. Handles mechanical analysis (file-to-crate mapping, concern classification, coupling
  detection), proposes a decomposition plan, requires explicit approval, then executes the split
  (creates branches, stages subsets, runs taskit check compile per split, optionally opens PRs). Trigger
  on "decompose this PR", "split this branch", "this diff is too big", "break this into smaller
  PRs", "make this reviewable", "/decompose", or any request to split a large changeset into
  independent units.
requires: []
next: []
allowed-tools: Bash, Read, Glob, Grep
---

# godmode:decompose — Diff Decomposition Agent

Splits a large diff or branch into independent, reviewable units. Mechanical analysis runs first
and is fully verifiable. LLM judgment is used only where the mechanical analysis is ambiguous —
and always requires explicit user approval before any git mutations occur.

## Helpers

| File                             | Purpose                                                                         |
| -------------------------------- | ------------------------------------------------------------------------------- |
| `helpers/analyze-diff.nu`        | Mechanical analysis: file→crate map, concern classification, coupling detection |
| `helpers/split-branch.nu`        | Branch creation, file staging, per-split verification loop                      |
| `helpers/decompose-proptest.rs`  | Property tests for grouping and coverage invariants                             |
| `references/split-strategies.md` | Decision guide for ambiguous grouping calls                                     |

## Reference files

Read the file named in a row before doing the step in its right column. These files are not
loaded automatically.

| File                                         | Read before                                                                              |
| -------------------------------------------- | ---------------------------------------------------------------------------------------- |
| `references/analysis-input-and-mechanics.md` | Step 0: choosing the input diff and recording the source branch, base, and changed files |
| `references/analysis-mechanics-step-1.md`    | Step 1: running the analyzer on the input diff, or reading its coupling warnings         |
| `references/review-and-plan.md`              | Step 2–3: judging groupings, naming splits, presenting the plan for approval             |
| `references/execute-splits.md`               | Step 4: running the splitter after approval, and the coverage check                      |
| `references/open-prs.md`                     | Step 5: opening PRs for the split branches, only when the user asks                      |
| `references/split-strategies.md`             | a grouping call is ambiguous and needs decision heuristics                               |

---

## Step 6 — Report

```text
Decomposition complete: <source-branch> → N splits

Split 1: <name> | <sha> | PR #<N> <url>
Split 2: <name> | <sha> | PR #<N> <url>
...

Coverage: all N changed files accounted for.
Orphaned: none | <list if any>

Source branch <source-branch> was not modified.
```

If any split failed verification, list it as `FAILED` with the failure summary. The source branch
is always intact regardless.

---

## Guardrails

- **Source branch is read-only.** Never commit to it, never delete it. All splits branch from base.
- **Plan approval is required.** No git mutations before the user says go.
- **Sequential verification.** Each split must pass `taskit check compile` + `nextest` before the next
  begins. A failing split halts the process — do not skip forward.
- **Coverage check.** After all splits, verify every changed file appears in exactly one split.
  Files in multiple splits or no split are bugs.
- **Coupling warnings are not blockers** — but they must be shown. The user decides whether to
  merge the affected splits or accept the dependency.
- **Dry-run by default** on ambiguous inputs. If uncertain what the user wants decomposed, ask
  rather than acting.
