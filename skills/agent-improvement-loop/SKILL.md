---
name: "godmode:agent-improvement-loop"
description: >
  Runs the agent improvement loop: collect traces → human+LLM feedback → generate evals →
  HALO diagnosis → Codex handoff → harness update. Use when you want to systematically
  improve an agent's behavior from observed evidence. Triggers on "run the improvement loop",
  "improve agent from traces", "agent feedback loop", "what should I fix in the agent".
requires: []
next: []
---

# Agent Improvement Loop

Connects observed agent behavior back to code changes through a structured seven-phase cycle.
Human judgment enters at the feedback phase and compounds through every subsequent step.

```text
SDK traces → Human+LLM feedback → Promptfoo evals → HALO diagnosis
    → Codex handoff → [Automation heartbeat] → Harness update
```

---

<loop>

<phase id="1" name="sdk-traces" label="Observed behavior">

## Phase 1: SDK Traces — Collect Observed Behavior

Goal: surface raw evidence of what the agent actually did.

**Default adapter (coursers/crs):** run the commands in `references/phase-1-trace-commands.md`.

**Generic adapter:** point at any trace source that produces structured records:

- OpenAI SDK traces (JSONL)
- LangSmith runs
- Promptfoo result files
- Custom session logs

**Output of this phase:**

- A list of agent actions (commands run, tools called, outputs produced)
- Failure cases: non-zero exits, rule blocks, unexpected outputs
- Frequency counts: which behaviors repeat

Save raw evidence to `.ctx/_WORKING_DIR/traces-<date>.json`.

</phase>

<phase id="2" name="human-llm-feedback" label="Interpret traces">

## Phase 2: Human + LLM Feedback — Interpret Traces

Goal: turn raw traces into labeled signal (good behavior / bad behavior / missing behavior).

**Steps:**

1. Present the top 10 most frequent unhandled or failing behaviors from Phase 1.
2. For each, ask: **block, allow, rewrite, or ignore?**
3. Cluster by pattern — behaviors that share a root cause go in the same cluster.
4. Label each cluster with:
   - `type`: `false-positive` | `missing-rule` | `wrong-message` | `threshold-tuning` | `new-behavior`
   - `severity`: `P1` (breaks workflow) | `P2` (annoying) | `P3` (cosmetic)
   - `evidence_count`: how many times seen

**LLM assist prompt (run inline):** use the prompt in `references/phase-2-feedback-prompt.md`,
with the trace samples filled in.

**Output of this phase:**

- `feedback.json` — labeled clusters, one per finding

</phase>

<phase id="3" name="promptfoo-evals" label="Generate + validate">

## Phase 3: Promptfoo Evals — Generate and Validate

Goal: turn feedback clusters into reproducible test cases that can gate future changes.

**For each cluster from Phase 2, generate an eval case** using the template in
`references/phase-3-eval-cases.md`. Save it to `.ctx/_WORKING_DIR/evals-<date>.yaml`.

**Run evals:** the promptfoo command and the shell fallback (for when promptfoo is not
available) are in `references/phase-3-eval-cases.md`.

**Output of this phase:**

- Passing evals = confirmed good behavior (regression tests)
- Failing evals = confirmed bad behavior (targets for Phase 4)

</phase>

<phase id="4" name="halo-diagnosis" label="Rank changes">

## Phase 4: HALO Diagnosis — Rank Changes

Goal: prioritize which changes to make based on impact × confidence × effort.

HALO = **H**igh-impact, **A**ctionable, **L**ow-effort, **O**bservable.

**Score each failing eval cluster** with the formula in `references/phase-4-halo-scoring.md`.

**Rank and emit a change list** in the JSON shape given in the same file.

**Output of this phase:**

- `diagnosis.json` — ranked change list with action, target file, and eval IDs

</phase>

<phase id="5" name="codex-handoff" label="Evidence to code">

## Phase 5: Codex Handoff — Evidence to Code

Goal: package the ranked change list into a handoff that Codex (or any coding agent) can
execute without further context.

**Handoff document format:** fill the template in `references/phase-5-handoff-template.md`.

**Save to:**

```text
.ctx/HANDOFF.agent-improvement.<agent-name>.yaml
```

**Or use `hj` if available:**

```nu
Read `.ctx/_WORKING_DIR/diagnosis.json`, then pass its contents to `hj handoff --notes`.
```

**Output of this phase:**

- Handoff file ready for Codex pickup

</phase>

<phase id="6" name="automation-heartbeat" label="Detect new handoffs and trigger Codex" optional="true">

## Phase 6: Automation + Heartbeat (optional)

Goal: remove the human from the loop for routine low-risk changes by automating handoff
detection and Codex triggering.

**Heartbeat script (Nushell cron or launchd):** see `references/phase-6-heartbeat-script.md`.

**Gate conditions before auto-triggering:**

- Change type must be `add-rule`, `update-message`, or `add-exception` (not `new-behavior`)
- HALO score must be ≥ 3.0
- All eval IDs in the handoff must exist and have a baseline result

**Output of this phase:**

- Changes triggered automatically without human prompt

</phase>

<phase id="7" name="harness-update" label="PR + validation">

## Phase 7: Harness Update — PR + Validation

Goal: validate that the changes work, the evals pass, and commit the result.

**Validation steps:** run the four checks in `references/phase-7-validation-commands.md`
(evals, `crs validate`, `taskit test run`, and the hook smoke test).

**On all green:**

```text
git branch --show-current   # MUST verify — stop if output is "main"
git add <changed files>
git commit -m "fix(agent): <cluster_id> — <one-line diagnosis>"
git push
```

**Update the harness:**

- Move passing evals into the permanent test fixture directory
- Add them to CI if not already there
- Archive the handoff as completed (`hj done` or rename to `.completed`)

**Loop back:**

- The next trace collection (Phase 1) will confirm the fix held in production

</phase>

</loop>

---

## Running the Loop

The full and partial run modes, and the invocation flags, are in `references/running-the-loop.md`. Read it before starting a run.

---

## Standalone implementation

The standalone updog workspace and its `ail` binary are described in `references/standalone-implementation.md`. Read it before installing or running `ail`.

---

## Adapters

<adapter id="coursers" default="true">

Coursers / crs is the default adapter. Its phase-by-phase commands and profile isolation rules are in `references/adapters.md`.

</adapter>

<adapter id="generic">

For any other trace source, the required record fields and the eval-harness swap are in `references/adapters.md`.

</adapter>
