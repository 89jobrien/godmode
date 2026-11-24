---
name: "rustqual-workspace"
description: >
  rustqual guidance for Rust workspaces with multiple crates (lib + binary, lib + CLI,
  multi-crate workspace). Use when rustqual findings don't make sense in a workspace
  context — DEAD_CODE on pub API, TQ_UNTESTED on CLI dispatch, score mismatch between
  workspace-level and per-crate scans, or ORPHAN_SUPPRESSION after adding qual:allow(dry).
  Triggers on: "rustqual workspace", "dead code false positive", "cross-crate",
  "lib and binary crate", "qual:allow dry not working", "ORPHAN_SUPPRESSION",
  or any rustqual scenario involving pub API used across crate boundaries.
color: orange
model: inherit
---

# rustqual-workspace

Workspace-specific guidance for rustqual. rustqual operates per-crate — it cannot
trace cross-crate call graphs. This creates predictable false positives in workspaces
that require configuration rather than code changes.

For full rustqual reference (finding codes, CLI flags, config schema, quality loop),
see the `rustqual` skill. This skill covers only the workspace-specific patterns.

## Reference Files

Read the file named in a row before doing the task in its right column. These files are not
loaded automatically.

| File                                             | Read before                                                                                       |
| ------------------------------------------------ | ------------------------------------------------------------------------------------------------- |
| `references/workspace-toml-examples.md`          | writing a `rustqual.toml` for a lib+binary, multi-crate, cfg-gated, or integration-test workspace |
| `references/workspace-toml-template.md`          | creating or editing the workspace-level `rustqual.toml` or a per-crate override                   |
| `references/pattern-dead-code-pub-api.md`        | a DEAD_CODE finding on a `pub` function used by another crate                                     |
| `references/pattern-tq-untested-cli-dispatch.md` | a TQ_UNTESTED finding on `main`, `run`, or a CLI dispatch function                                |
| `references/pattern-score-mismatch.md`           | comparing a workspace score against per-crate scores, or saving a baseline                        |
| `references/pattern-lib-untested.md`             | a TQ_UNTESTED finding on a function in a library crate                                            |
| `references/triage-decision-tree.md`             | a finding's fix is not clear from its code location alone                                         |
| `helpers/triage.nu`                              | running per-crate triage: `nu helpers/triage.nu [--crate <path>] [--json]`                        |

`helpers/triage.nu` runs rustqual on each crate, annotates findings with recommended
actions (fix vs. config vs. suppress), and groups output by action type.

## The Core Limitation

rustqual analyzes one crate at a time. It cannot see:

- Functions in `lib-crate` called only by `binary-crate`
- Functions called by integration tests in `tests/`
- Functions called by other workspace members

This produces **structural false positives** that are correct findings within a single
crate but wrong in the workspace context.

## Symptom index

| Finding or symptom                                       | Read                                             |
| -------------------------------------------------------- | ------------------------------------------------ |
| `DEAD_CODE (testonly)` on a `pub` function               | `references/pattern-dead-code-pub-api.md`        |
| `ORPHAN_SUPPRESSION` after `qual:allow(dry)`             | `references/pattern-dead-code-pub-api.md`        |
| `TQ_UNTESTED` on `main`, `run`, `dispatch_*` in a binary | `references/pattern-tq-untested-cli-dispatch.md` |
| Workspace score does not match per-crate scores          | `references/pattern-score-mismatch.md`           |
| `TQ_UNTESTED` on a lib function                          | `references/pattern-lib-untested.md`             |
