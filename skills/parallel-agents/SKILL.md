---
name: parallel-agents
description: >
  Use when facing 2 or more independent tasks that have no shared state or sequential
  dependencies between them. Triggers on multiple failing tests from different root causes,
  independent crate implementations, or independent task chains in GODMODE.tasks.yaml.
---

# Parallel Agents

Dispatch one agent per independent domain. Let them work concurrently.

## When to Use

Use parallel dispatch when:

- Multiple task chains in `.ctx/GODMODE.tasks.yaml` have no shared `depends_on`
- Multiple crates need implementation and they don't share types
- Multiple test failures originate from different subsystems
- Each problem is fully self-contained

Do NOT use when:

- Tasks share state or cross-crate types
- One task produces types consumed by another
- You're still in exploratory debugging (unclear scope)

## Dispatch Protocol

### Step 1: Identify independent units

From `.ctx/GODMODE.tasks.yaml`, find all tasks where `depends_on` is empty or all
dependencies are `done`. Group into chains. Chains with no shared nodes are independent.

For crate-based work: each crate is an independent unit if it doesn't import from
another in-progress crate.

### Step 2: Prepare focused prompts

Each agent prompt must be:

- **Self-contained**: all context the agent needs, inline. No "see the other agent".
- **Scoped**: one crate or one problem domain only.
- **Explicit**: exact test names, file paths, task descriptions.
- **Constrained**: "Do NOT modify files outside `crates/<crate>/`."

Template:

```
You are implementing tasks for crate: <CRATE>

Tasks:
<TASK_LIST with exact descriptions>

Workflow (follow exactly):
1. Read crates/<CRATE>/src/ files relevant to the tasks.
2. For each task:
   a. Write a FAILING test. Run: cargo nextest run -p <CRATE> -- <test>. Confirm FAIL.
   b. Implement minimum code to pass. Run: cargo nextest run -p <CRATE>. All green.
   c. Run: cargo clippy -p <CRATE> -- -D warnings. Fix all warnings.
   d. Commit: git commit -m "feat(<CRATE>): <summary>"
3. If stuck after 3 attempts: write BLOCKED.md at repo root. Stop.
4. Final check: cargo nextest run -p <CRATE> && cargo clippy -p <CRATE> -- -D warnings

Report: tasks completed, tasks blocked, commit SHAs.
```

### Step 3: Initialize wave state

Before dispatching, write `.ctx/wave-status.json`:

```json
{
  "wave": 1,
  "agents": {
    "<crate-or-domain>": {
      "status": "pending",
      "branch": "",
      "commits": []
    }
  }
}
```

Each agent updates its own entry on completion. The orchestrator reads this file to track
convergence without polling agent sessions.

### Step 4: Dispatch concurrently

Cap at **5 concurrent agents**. If more than 5 independent units exist, queue the rest
and dispatch each as a slot frees.

Pass explicit tool lists — agents do NOT inherit Bash permissions from parent:

```
allowedTools: Read, Write, Edit, Bash, Grep, Glob
```

Instruct each agent to update wave state on finish:

```bash
# agent writes on completion:
# status: "done" | "blocked"
# branch: output of git branch --show-current
# commits: list of SHAs from git log --oneline -3
```

### Step 5: Integrate results

After all agents report:

1. Read `.ctx/wave-status.json` — verify no agent has `status: "pending"`.
2. Any agent with empty `commits` list has not finished — do not proceed.
3. Verify each agent committed (`git log --oneline -5`). Empty log = incomplete.
4. Run full workspace suite:
   ```bash
   cargo nextest run --workspace
   cargo clippy --workspace -- -D warnings
   ```
5. Merge each branch sequentially with `--no-ff` (never octopus-merge). Integration failures
   (cross-crate) are fixed in the orchestrator session — do not spawn another agent layer.
6. Update `.ctx/GODMODE.tasks.yaml`: mark completed tasks `done`, blocked tasks `blocked`.
7. Archive wave state: `mv .ctx/wave-status.json .ctx/wave-<N>-complete.json`

## Guardrails

- **Crate isolation is the unit of parallelism.** Never dispatch two agents for the same
  crate simultaneously.
- **BLOCKED.md stops the agent.** 3 failed attempts → write BLOCKED.md → stop. Do not
  retry with identical parameters.
- **Agents don't inherit permissions.** Always pass `allowedTools` explicitly.
- **Never octopus-merge.** If agents diverge, cherry-pick sequentially.
- **Each agent must verify `git branch --show-current` before every commit.** If it
  returns `main`, stop — do not commit to main directly.

## Additional Resources

- **`references/dispatch-protocol.md`** — independence test, worktree setup, integration steps, BLOCKED.md protocol
- **`helpers/agent-prompt-template.md`** — fill-in-the-blank prompt for each dispatched agent
