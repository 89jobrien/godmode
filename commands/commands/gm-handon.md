---
name: handon
allowed_tools:
  - Bash
  - Read
  - Glob
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

Run session-start triage and orient to outstanding work.

1. Run: godmode handon
2. Run: godmode task next
3. Check .ctx/godmode/traces/trace.jsonl for any skill.error or agent.blocked events from
   the last session using skills/observability-as-infrastructure/helpers/session-summary.nu
4. Report: running tasks, next runnable task(s), any unresolved failures from last session.
