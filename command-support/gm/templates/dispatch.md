## Rules

- Cap parallel subagents at 5 concurrent.
- Each subagent must run `git branch --show-current` before every commit.
  If on main, STOP — do not commit to main directly.
- Worktree subagents commit only on their assigned branch. The parent dispatcher
  exclusively owns sequential merges and worktree cleanup.
- Never use octopus merges — merge sequentially, one branch at a time.
- After each agent completes, verify commits exist: `git log --oneline -3`.
  A HANDOFF with `commits: []` is incomplete.
- If stuck after 3 attempts: write BLOCKED.md and stop.
- Never use `--no-verify` in subagent git operations.
- If `gh` auth fails, log to `.ctx/godmode/pending-manual.txt` and continue.
  Do NOT retry auth — tell the user to run `gh auth login`.
