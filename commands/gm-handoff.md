---
description: "Run session-end validation and write a handoff record."
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

Run session-end validation and write a handoff record.

1. Run: godmode handoff
2. Run `nu ($env.HOME | path join ".agents" "skills" "observability-as-infrastructure" "helpers" "trace-stats.nu")` to summarise
   skill durations, agent convergence, and decisions made this session.
3. If any tasks are still running, report them — do not silently ignore.
4. Report: tasks completed, tasks blocked, any open issues requiring follow-up.
