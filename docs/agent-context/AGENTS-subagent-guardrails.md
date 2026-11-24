# Subagent guardrails

Read this before dispatching subagents or running parallel worktree work.

When dispatching subagents:

- Each subagent must run `git branch --show-current` immediately before every `git commit`.
  If the answer is `main`, STOP — do not commit to main directly.
- Worktree subagents commit and report their branch/SHA without integrating or removing the
  worktree. The orchestrator delegates integration and cleanup to wave integration.
- After subagents complete, verify their changes were committed (`git log --oneline -3`).
  A HANDOFF with `commits: []` is incomplete.
- Merge parallel branches sequentially with `git merge --no-ff`; never cherry-pick or use an
  octopus merge.
- Cap parallel subagents at 5 concurrent to avoid API rate limits.
- Never use `--no-verify` in subagent git operations.
- After 3 failed attempts, write `BLOCKED.md` and stop before escalating.
