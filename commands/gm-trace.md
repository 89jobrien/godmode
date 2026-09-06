---
description: "Query the observability trace for this request"
allowed-tools:
  - Bash
  - Read
---

## Rules

- Default to read-only. Do not modify files unless the command explicitly
  requires it.
- Session trace lives at `.ctx/godmode/traces/trace.jsonl`.
- Task state lives at `.ctx/godmode/tasks.yaml`.
- Scratch dir is `.ctx/godmode/_WORKING_DIR/`.
- Report findings in plain text. Flag any `agent.blocked` or `skill.error`
  events prominently.

Query the observability trace for this request: $ARGUMENTS
Accept only an empty value, `failures`, `stats`, `summary`, or `tail N` where N is an integer
from 1 through 1000. Default empty input to `tail 20`; reject anything else.
Run the appropriate helper based on what's needed:

- Last N events: `nu ($env.HOME | path join ".agents" "skills" "observability-as-infrastructure" "helpers" "trace-tail.nu") --n N`
- Failures only: `nu ($env.HOME | path join ".agents" "skills" "observability-as-infrastructure" "helpers" "trace-failures.nu")`
- Durations + agents: `nu ($env.HOME | path join ".agents" "skills" "observability-as-infrastructure" "helpers" "trace-stats.nu")`
- Cross-session: `nu ($env.HOME | path join ".agents" "skills" "observability-as-infrastructure" "helpers" "session-summary.nu")`
  Report findings in plain text. Flag any agent.blocked or skill.error events prominently.
