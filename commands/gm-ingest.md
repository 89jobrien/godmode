---
name: ingest
allowed-tools:
- Bash
- Read
- Glob
- Grep
max-turns: 15
---
## Rules

- Default to read-only. Do not modify files unless the command explicitly
  requires it.
- Session trace lives at `.ctx/godmode/sessions/YYYY-MM-DD.jsonl`.
- Task state lives at `.ctx/godmode/tasks.yaml`.
- Scratch dir is `.ctx/_WORKING_DIR/`.
- Report findings in plain text. Flag any `agent.blocked` or `skill.error`
  events prominently.


Invoke godmode:ingest for the optional plan path in `$ARGUMENTS`.
Treat `$ARGUMENTS` as a literal file path, never as shell syntax.
If no path is supplied, select the most recently modified Markdown file under
`.ctx/godmode/plans/`. Validate the plan, capture the graph before and after ingestion, and
report the exact path, assigned IDs, added/skipped counts, and next runnable work. Never clear
the graph or edit `.ctx/godmode/tasks.yaml` directly. On success, hand off to `/gm:tdd`.
