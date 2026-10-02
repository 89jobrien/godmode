---
name: handon
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


Produce a session-start status update. This command is a readout: report what the commands
printed, and do not infer or reconstruct state they did not print.

1. Run: godmode handon
2. Run: godmode task next
3. Check .ctx/godmode/traces/trace.jsonl for skill.error or agent.blocked events left by the
   previous session, using skills/observability-as-infrastructure/helpers/session-summary.nu

Report with these sections. Omit a section only when nothing was printed for it.

- State — done / running / pending / blocked counts.
- Waiting on — the active pipeline and the step it is blocked at.
- Next — the single next runnable task, by ID and title.
- Unresolved — errors or blocked agents carried over from the last session.
- Tree — dirty or clean, naming files when dirty.

Note: `godmode handon` reports task-graph state only. It does not query doob — use
`godmode task pull` or `doob todo list` for pending todos.
