---
name: trace
allowed_tools:
  - Bash
  - Read
max_turns: 5
---

## Rules

- Default to read-only. Do not modify files unless the command explicitly
  requires it.
- Session trace lives at `.ctx/sessions/YYYY-MM-DD.jsonl`.
- Task state lives at `.ctx/godmode/tasks.yaml`.
- Scratch dir is `.ctx/_WORKING_DIR/`.
- Report findings in plain text. Flag any `agent.blocked` or `skill.error`
  events prominently.

Query the observability trace for this session.
Run the appropriate helper based on what's needed:

- Last N events: nu skills/observability-as-infrastructure/helpers/trace-tail.nu [--n 20]
- Failures only: nu skills/observability-as-infrastructure/helpers/trace-failures.nu
- Durations + agents: nu skills/observability-as-infrastructure/helpers/trace-stats.nu
- Cross-session: nu skills/observability-as-infrastructure/helpers/session-summary.nu
  Report findings in plain text. Flag any agent.blocked or skill.error events prominently.
