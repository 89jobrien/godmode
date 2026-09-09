---
description: "Run session-start triage and orient to outstanding work."
allowed-tools:
  - Bash
  - Read
  - Glob
---

## Rules

- Default to read-only. Do not modify files unless the command explicitly
  requires it.
- Session trace lives at `.ctx/godmode/traces/trace.jsonl`.
- Task state lives at `.ctx/godmode/tasks.yaml`.
- Scratch dir is `.ctx/godmode/_WORKING_DIR/`.
- Report findings in plain text. Flag any `agent.blocked` or `skill.error`
  events prominently.

Run session-start triage and orient to outstanding work.
1. Run: godmode handon
2. Run: godmode task next
3. Check the last traced session using `godmode trace summary --sessions 1 --previous`.
4. Report: running tasks, next runnable task(s), any unresolved failures from last session.
