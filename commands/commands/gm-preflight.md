---
name: preflight
allowed_tools:
  - Bash
  - Read
  - Edit
  - Glob
  - Grep
max_turns: 20
---

## Rules

- Default to read-only. Do not modify files unless the command explicitly
  requires it.
- Session trace lives at `.ctx/sessions/YYYY-MM-DD.jsonl`.
- Task state lives at `.ctx/godmode/tasks.yaml`.
- Scratch dir is `.ctx/_WORKING_DIR/`.
- Report findings in plain text. Flag any `agent.blocked` or `skill.error`
  events prominently.

Run pre-flight checks before pushing or starting parallel issue resolution.

1. Verify git branch: run `git branch --show-current`. If on main, STOP and report.
2. Run `cargo fmt --check --all`. Fix any formatting issues found.
3. Run `cargo clippy --workspace -- -D warnings`. Fix any warnings found.
4. Run `cargo nextest run --workspace`. Fix any test failures found.
5. Verify gh CLI auth: run `gh auth status`. If auth fails, report immediately — do NOT proceed
   with any gh operations. Tell the user to run `gh auth login` manually.
6. Report pass/fail for each check. Only confirm "ready to push" when all 5 pass clean.
