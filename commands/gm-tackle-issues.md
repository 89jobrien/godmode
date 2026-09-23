---
name: tackle-issues
allowed-tools:
- Bash
- Read
- Edit
- Write
- Glob
- Grep
max-turns: 60
---
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


Fetch open GitHub issues, group into independent units, and dispatch parallel agents.
Follow godmode:tackle-issues exactly:
1. Fetch issues: gh issue list --state open --limit 20 --json number,title,body,labels
2. Classify each as independent or dependent (same crate + overlapping files = dependent).
3. Present grouping (max 5 slots) and wait for go/no-go before proceeding.
4. Run skills/tackle-issues/helpers/setup-worktrees.nu <issue-numbers...>
5. Dispatch one godmode-crate-agent per slot with a self-contained prompt (see SKILL.md).
6. After agents complete, run skills/tackle-issues/helpers/integrate-branches.nu <issue-numbers...>
7. Close issues with gh issue close <N> --comment "Implemented in <sha>."
Never dispatch two agents to the same crate simultaneously.
Never use --no-verify. Cap at 5 concurrent agents.
