---
name: "rustqual"
description: >
  Rust code quality analysis and improvement using the rustqual CLI. Use when
  the user asks to "run rustqual", "check quality", "fix rustqual findings",
  "improve quality score", "what does rustqual say", or when you see rustqual
  output pasted into the conversation. Also use when configuring rustqual.toml,
  adding qual:allow suppressions, interpreting IOSP/DRY/SRP/TQ findings, or
  working toward a quality score target. Triggers on any mention of rustqual,
  quality score percentages, or finding codes like TQ_UNTESTED, VIOLATION,
  DEAD_CODE, SRP_STRUCT, SRP_MODULE, BOILERPLATE, or TQ_NO_SUT.
color: orange
model: inherit
---

# rustqual

rustqual is a six-dimension Rust code quality analyzer. It scores codebases
on IOSP, Complexity, DRY, SRP, Coupling, Test Quality, and Architecture,
producing a weighted quality score from 0-100%.

## Quick Reference

```bash
rustqual <path>                    # Analyze a crate or workspace
rustqual <path> --no-fail          # Don't exit 1 on findings
rustqual <path> --suggestions      # Show refactoring hints for IOSP
rustqual <path> --json             # Machine-readable output
rustqual <path> --diff             # Only analyze changed files vs HEAD
rustqual --init                    # Generate default rustqual.toml
```

## The Six Dimensions

| Dimension    | What it measures                                      | Weight |
| ------------ | ----------------------------------------------------- | ------ |
| IOSP         | Integration/Operation separation — no mixing          | 0.22   |
| Complexity   | Cognitive/cyclomatic complexity, nesting, length      | 0.18   |
| DRY          | Duplicate code, dead code, boilerplate                | 0.13   |
| SRP          | Single responsibility — cohesion, module size, params | 0.18   |
| Coupling     | Fan-in/out, instability, circular deps                | 0.09   |
| Test Quality | Coverage, naming, untested functions                  | 0.10   |
| Architecture | Layer rule compliance                                 | 0.10   |

## Finding Codes

Each finding has a code that tells you what category it belongs to and
how to fix it:

The IOSP and test-quality codes are in `references/finding-codes-iosp-and-test-quality.md`. The DRY and SRP codes are in `references/finding-codes-dry-and-srp.md`. Read the file that covers a code before fixing or suppressing it.

## Inline Suppressions

The suppression syntax, the dimension keys, and the gotchas are in `references/inline-suppressions.md`. Read it before adding a `qual:allow` comment.

## Configuration (rustqual.toml)

Generate a tailored config with `rustqual --init`, then adjust it. The file
covers ignore lists, IOSP tuning flags, and the complexity, duplicates, SRP,
and weights tables.

Read `references/rustqual-toml-example.md` before writing or editing
`rustqual.toml`. It has the full annotated example.

## Workflow: Improving a Quality Score

The four-step priority order for fixing findings is in `references/improvement-workflow.md`. Read it when asked to fix rustqual findings or raise the score.

## Common Pitfalls

- **Splitting functions to fix IOSP can increase TQ_UNTESTED**: If you
  extract 5 sub-functions from one function, you now have 5 untested
  functions instead of 1. Only split when the resulting functions are
  independently testable.

- **Don't chase 100% by disabling checks**: Use `detect_dead_code = false`
  and `ignore_functions` only for documented false positives. Document
  the reasoning in config comments.

- **Read transcripts, not just scores**: A score increase from refactoring
  that makes code harder to read is not an improvement. The score serves
  the code, not the other way around.

## Baselines and CI Integration

Baseline commands, the CI gate, and the workspace versus per-crate scan scope are in `references/baselines-and-scan-scope.md`. Read it before saving a baseline or comparing scores.

## Autonomous Quality Loop

When asked to "improve quality" or "run the quality loop", read
`references/autonomous-quality-loop.md` first. It defines the MEASURE and FIX
phases with their commands, scan scope, and exit criteria. The VERIFY and COMMIT
phases are in `references/autonomous-quality-loop-verify-and-commit.md`. Run all
four phases in order and do not skip MEASURE, because every
later comparison depends on the baseline it saves.

## cfg-gated Code (Kani, test utilities)

`#[cfg(kani)]` modules and `#[cfg(any(test, feature = "testutil"))]` code
are invisible to rustqual's dead-code analysis. Read
`references/cfg-gated-code.md` before choosing between moving that code to
its own file and listing it in `exclude_files`, or turning off dead-code
detection.

## Reference Files

- **`references/finding-codes-iosp-and-test-quality.md`** — IOSP, TQ_UNTESTED, and TQ_NO_SUT codes with fix patterns. Read before fixing or suppressing one of these findings.
- **`references/finding-codes-dry-and-srp.md`** — DEAD_CODE, BOILERPLATE, SRP_STRUCT, and SRP_MODULE codes with fix patterns. Read before fixing one of these findings or changing `detect_dead_code`.
- **`references/inline-suppressions.md`** — `qual:allow` syntax, dimension keys, and gotchas. Read before adding a suppression.
- **`references/improvement-workflow.md`** — Priority order for fixing findings. Read when asked to fix findings or raise the score.
- **`references/baselines-and-scan-scope.md`** — Baseline and CI gate commands, and scan scope. Read before saving a baseline or comparing scores.

- **`references/cli-reference.md`** — Full CLI options, output formats,
  config file schema, exit codes, CI examples
- **`references/fix-patterns.md`** — Concrete before/after Rust code
  examples for every finding type
- **`references/workflow.md`** — Step-by-step walkthrough of a typical
  improvement session from first run to 100%, with progression table
- **`references/rustqual-toml-example.md`** — Annotated `rustqual.toml`
  example. Read before writing or editing the config.
- **`references/autonomous-quality-loop.md`** — The MEASURE and FIX phases
  with commands and exit criteria. Read before running "improve quality" or
  "run the quality loop".
- **`references/autonomous-quality-loop-verify-and-commit.md`** — The VERIFY and
  COMMIT phases. Read after FIX, before verifying or committing the baseline.
- **`references/cfg-gated-code.md`** — How to handle `#[cfg(kani)]` and
  cfg-gated test utilities in dead-code findings. Read when a finding
  involves cfg-gated code.
