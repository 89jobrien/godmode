## Rules

- Default to read-only. Do not modify files unless the command explicitly
  requires it.
- Session trace lives at `.ctx/godmode/traces/trace.jsonl`.
- Task state lives at `.ctx/godmode/tasks.yaml`.
- Scratch dir is `.ctx/godmode/_WORKING_DIR/`.
- Report findings in plain text. Flag any `agent.blocked` or `skill.error`
  events prominently.
