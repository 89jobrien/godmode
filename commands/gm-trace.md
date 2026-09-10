---
description: "Query and summarize observability trace events for the current request."
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
Run the appropriate built-in query:
- Last N events:      `godmode trace tail --n N`
- Failures only:      `godmode trace failures`
- Durations + agents: `godmode trace stats`
- Cross-session:      `godmode trace summary`
Report findings in plain text. Flag any agent.blocked or skill.error events prominently.
