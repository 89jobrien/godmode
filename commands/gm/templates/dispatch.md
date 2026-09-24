## Rules

- Cap parallel subagents at 5 concurrent.
- Each subagent must run `git branch --show-current` before every commit.
  If on main, STOP — do not commit to main directly.
- Worktree subagents commit and report their branch/SHA without integrating or
  removing the worktree.
- The orchestrator delegates integration to wave integration, which merges
  branches one at a time with `git merge --no-ff`. Never cherry-pick or use an
  octopus merge for parallel branches.
- After each agent completes, verify commits exist: `git log --oneline -3`.
  A HANDOFF with `commits: []` is incomplete.
- After 3 failed attempts, write `BLOCKED.md` and stop.
- Never use `--no-verify` in subagent git operations.
- If `gh` auth fails, log to `.ctx/pending-manual.txt` and continue.
  Do NOT retry auth — tell the user to run `gh auth login`.
