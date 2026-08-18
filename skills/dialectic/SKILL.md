---
name: "godmode:dialectic"
description: >
  Adversarial-reasoning pipeline for contentious decisions. Use when weighing tradeoffs,
  choosing between competing approaches, or wanting structured adversarial scrutiny before
  committing to a direction. Frames a question, dispatches Advocate/Skeptic/Alternative
  proposers in parallel, then synthesizes into a single reconciled answer.
requires: []
next: []
---

# Dialectic — Adversarial Synthesis

Run a structured adversarial-reasoning pass on a decision before committing to an approach.

## When to use

- A decision is contentious or has real tradeoffs (not a clear-cut technical fix)
- You want deliberate scrutiny — a case for the obvious answer, a case against it, and a
  fundamentally different framing — before proceeding
- Architecture choices, "should we do X or Y", policy/process decisions, anything where being
  wrong is expensive enough to warrant three independent takes

Not for: straightforward bug fixes, tasks with one clearly correct implementation, or anything
time-sensitive where a single well-reasoned answer suffices.

## Workflow

Dispatch `dialectic-orchestrator` via the Agent tool with the question or decision to analyze.
The orchestrator handles the rest of the pipeline itself:

1. Frames the question into adversarial roles
2. Dispatches `dialectic-advocate`, `dialectic-skeptic`, and `dialectic-alternative` in parallel
3. Passes all three proposals to `dialectic-synthesizer`, which reconciles them into one answer
   with rationale, open questions, and rejected positions

Do not dispatch the four proposer/synthesizer agents directly — always go through
`dialectic-orchestrator` so the pipeline stays coordinated.
