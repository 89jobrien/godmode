---
name: handoff
allowed-tools:
- Bash
- Read
- Glob
max-turns: 5
---
## Rules

- Default to read-only. Do not modify files unless the command explicitly
  requires it.
- Session trace lives at `.ctx/godmode/sessions/YYYY-MM-DD.jsonl`.
- Task state lives at `.ctx/godmode/tasks.yaml`.
- Scratch dir is `.ctx/_WORKING_DIR/`.
- Report findings in plain text. Flag any `agent.blocked` or `skill.error`
  events prominently.


Produce a session-end status update. This command is a readout: report what the commands printed,
and do not infer or reconstruct state they did not print.

1. Run: godmode handoff
2. Run skills/observability-as-infrastructure/helpers/trace-stats.nu to summarise skill
   durations, agent convergence, and decisions made this session.
3. Surface every task still `running` — never silently drop one.

Report with these sections. Omit a section only when nothing was printed for it.

- Completed — what finished this session.
- Still running — every task left in `running`, by ID and title. Never report the session as
  clean while this is non-empty.
- Blocked — blocked tasks and the reason each is blocked.
- Tree — dirty or clean, naming files when dirty.
- Next — the single most useful next action.

Do not claim the work is finished while a task is still `running`.

`godmode handoff` writes a tracked `HANDOFF.md` at the repository root. It writes no external
handoff record. Task state remains in `.ctx/godmode/tasks.yaml`.
