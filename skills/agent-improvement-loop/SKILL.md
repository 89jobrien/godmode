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

```
SDK traces → Human+LLM feedback → Promptfoo evals → HALO diagnosis
    → Codex handoff → [Automation heartbeat] → Harness update
```

---

<loop>

<phase id="1" name="sdk-traces" label="Observed behavior">

## Phase 1: SDK Traces — Collect Observed Behavior

Goal: surface raw evidence of what the agent actually did.

**Default adapter (coursers/crs):**

```nu
# Recent session history for current project
crs discover --since 30 --format json

# Richer facet view with git context
crs insights --format json

# Block stats — which rules fired most
crs stats
```

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

**LLM assist prompt (run inline):**

```
Given these agent trace samples, identify:
1. Patterns that should be blocked but aren't
2. Patterns that are blocked but shouldn't be (false positives)
3. Block messages that are unhelpful or wrong
4. Threshold values that fire too early or too late

Traces:
<traces>{{traces}}</traces>

Respond as JSON: { clusters: [{ id, type, severity, evidence_count, sample, diagnosis }] }
```

**Output of this phase:**

- `feedback.json` — labeled clusters, one per finding

</phase>

<phase id="3" name="promptfoo-evals" label="Generate + validate">

## Phase 3: Promptfoo Evals — Generate and Validate

Goal: turn feedback clusters into reproducible test cases that can gate future changes.

**For each cluster from Phase 2, generate an eval case:**

```yaml
# .ctx/_WORKING_DIR/evals-<date>.yaml
description: "<cluster.id> — <cluster.diagnosis>"
providers:
  - id: exec
    config:
      command: "echo '<sample_command>' | coursers pre --profile test"

tests:
  - description: "should block <pattern>"
    assert:
      - type: javascript
        value: "output.includes('deny')"

  - description: "should NOT block <exception>"
    assert:
      - type: javascript
        value: "!output.includes('deny')"
```

**Run evals:**

```nu
promptfoo eval --config .ctx/_WORKING_DIR/evals-<date>.yaml --output .ctx/_WORKING_DIR/eval-results-<date>.json
```

**If promptfoo is not available**, generate shell-based smoke tests instead:

```nu
# Smoke test equivalent
echo '{"tool_name":"Bash","tool_input":{"command":"<sample>"}}' | coursers pre
```

**Output of this phase:**

- Passing evals = confirmed good behavior (regression tests)
- Failing evals = confirmed bad behavior (targets for Phase 4)

</phase>

<phase id="4" name="halo-diagnosis" label="Rank changes">

## Phase 4: HALO Diagnosis — Rank Changes

Goal: prioritize which changes to make based on impact × confidence × effort.

HALO = **H**igh-impact, **A**ctionable, **L**ow-effort, **O**bservable.

**Score each failing eval cluster:**

```
impact    = evidence_count × severity_weight  (P1=3, P2=2, P3=1)
confidence = promptfoo pass rate on existing cases (0.0–1.0)
effort     = estimated lines of change (1=<5, 2=5-20, 3=>20)
halo_score = (impact × confidence) / effort
```

**Rank and emit a change list:**

```json
{
  "changes": [
    {
      "rank": 1,
      "cluster_id": "no-sleep-in-codex",
      "type": "missing-rule",
      "halo_score": 4.2,
      "action": "add rule to ~/.config/coursers/profiles/codex/rules.json",
      "target_file": "~/.config/coursers/profiles/codex/rules.json",
      "evidence_count": 14,
      "eval_ids": ["eval-003", "eval-007"]
    }
  ]
}
```

**Output of this phase:**

- `diagnosis.json` — ranked change list with action, target file, and eval IDs

</phase>

<phase id="5" name="codex-handoff" label="Evidence to code">

## Phase 5: Codex Handoff — Evidence to Code

Goal: package the ranked change list into a handoff that Codex (or any coding agent) can
execute without further context.

**Handoff document format:**

```markdown
# Agent Improvement Handoff — <date>

## Context

<one paragraph: what agent, what loop iteration, what source of traces>

## Changes (ranked by HALO score)

### Change 1: <cluster_id>

- **File**: `<target_file>`
- **Action**: <add rule | update threshold | fix message | add exception>
- **Evidence**: <evidence_count> occurrences in last 30 days
- **Eval gate**: `<eval_id>` must pass after change
- **Spec**:
  <exact JSON/TOML/code block to add or modify>

### Change 2: ...
```

**Save to:**

```
.ctx/HANDOFF.agent-improvement.<agent-name>.yaml
```

**Or use `hj` if available:**

```nu
hj handoff --title "agent improvement loop: <agent>" --notes "$(cat .ctx/_WORKING_DIR/diagnosis.json)"
```

**Output of this phase:**

- Handoff file ready for Codex pickup

</phase>

<phase id="6" name="automation-heartbeat" label="Detect new handoffs and trigger Codex" optional="true">

## Phase 6: Automation + Heartbeat (optional)

Goal: remove the human from the loop for routine low-risk changes by automating handoff
detection and Codex triggering.

**Heartbeat script (Nushell cron or launchd):**

```nu
#!/usr/bin/env nu
# ~/.local/bin/agent-improvement-heartbeat.nu
# Run every N minutes via cron or launchd

let handoff_dir = $"($env.HOME)/dev/.ctx"
let pattern = "HANDOFF.agent-improvement.*.yaml"

let pending = (ls $"($handoff_dir)/($pattern)" | where modified > ((date now) - 1hr))

if ($pending | length) > 0 {
    for $f in $pending {
        # Trigger Codex with the handoff as context
        codex --context (open $f.name | to json) "Apply the changes in this handoff. Run evals after each change. Commit only if all evals pass."
    }
}
```

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

**Validation steps:**

```nu
# 1. Run the evals that were failing in Phase 3
promptfoo eval --config .ctx/_WORKING_DIR/evals-<date>.yaml

# 2. Run the full rule health check
crs validate [--profile <profile>]

# 3. Run workspace tests
cargo nextest run --workspace

# 4. Smoke test the hook pipeline
echo '{"tool_name":"Bash","tool_input":{"command":"<sample>"}}' | coursers pre [--profile <profile>]
```

**On all green:**

```
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

### Full loop (interactive)

Work through phases 1–7 in order. Present findings at each phase boundary and wait for
confirmation before proceeding to the next phase.

### Partial loop

- **Diagnose only**: run phases 1–4, emit `diagnosis.json`, stop
- **Eval-first**: skip phase 1, start from existing `feedback.json` at phase 3
- **Harness-only**: given a `diagnosis.json`, jump to phase 5 and produce the handoff

### Loop invocation

```
/agent-improvement-loop [--agent <name>] [--since <days>] [--phase <N>] [--dry-run]
```

| Flag        | Default         | Effect                                   |
| ----------- | --------------- | ---------------------------------------- |
| `--agent`   | current project | Which agent's traces to analyze          |
| `--since`   | 30              | Days of trace history to scan            |
| `--phase`   | 1               | Start from this phase                    |
| `--dry-run` | false           | Run through phases without writing files |

---

## Standalone implementation

The loop is also available as a standalone Rust workspace: **updog** (`~/dev/updog`,
https://github.com/89jobrien/updog).

- `agent-loop` crate — serializable types: `TraceRecord`, `FeedbackCluster`, `HaloScore`,
  `ChangeItem`, `Diagnosis`, `Handoff`
- `ail` binary — `ail run [--agent <name>] [--since <days>] [--phase <N>] [--dry-run]`

Install: `cargo install --path ~/dev/updog/crates/ail`

Phases 1, 4, 5 are fully implemented. Phases 2, 3 emit guided prompts and resume instructions.
Phases 6, 7 print next steps and run `crs validate`.

---

## Adapters

<adapter id="coursers" default="true">

### Coursers / crs adapter (default)

| Loop phase       | Command                                            |
| ---------------- | -------------------------------------------------- |
| Phase 1 traces   | `crs discover --since <N> --format json`           |
| Phase 1 insights | `crs insights --format json`                       |
| Phase 1 stats    | `crs stats`                                        |
| Phase 3 evals    | `echo '<payload>' \| coursers pre [--profile <p>]` |
| Phase 7 validate | `crs validate [--profile <p>]`                     |
| Phase 7 test     | `cargo nextest run --workspace`                    |

Profile isolation: pass `--profile <name>` to all coursers/crs commands to keep
test-profile state separate from production.

</adapter>

<adapter id="generic">

### Generic adapter

Replace Phase 1 with any trace source that emits structured JSON records with fields:

- `command` or `input` — what the agent did
- `exit_code` or `outcome` — success/failure
- `timestamp` — when it happened
- `session_id` (optional) — for grouping

Replace Phase 3 evals with any eval harness (promptfoo, pytest, custom shell scripts) that
can assert on agent output given a fixed input.

</adapter>
