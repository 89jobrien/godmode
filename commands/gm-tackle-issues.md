---
description: "Fetch open GitHub issues, group into independent units, and dispatch parallel agents."
allowed-tools:
  - Bash
  - Read
  - Edit
  - Write
  - Glob
  - Grep
  - Agent
---

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

Fetch open GitHub issues, group into independent units, and dispatch parallel agents.
Arguments: $ARGUMENTS (optional issue numbers; default: all open issues).
If non-empty, accept only whitespace-separated `N` or `#N` tokens. Reject all input if any
token is invalid, normalize to integer issue numbers, and pass each number as a separate argument.
Follow godmode:tackle-issues exactly:
1. Fetch issues: gh issue list --state open --limit 20 --json number,title,body,labels
2. Classify each as independent or dependent (same crate + overlapping files = dependent).
3. Present grouping (max 5 slots) and wait for go/no-go before proceeding.
4. Run `nu ($env.HOME | path join ".agents" "skills" "tackle-issues" "helpers" "setup-worktrees.nu")` with each
   validated issue number as a separate argument.
5. Dispatch one `godmode:gm-crate` per slot with a self-contained prompt (see SKILL.md).
6. After agents complete, the parent dispatcher runs
   `nu ($env.HOME | path join ".agents" "skills" "tackle-issues" "helpers" "integrate-branches.nu")` with each issue
   number as a separate argument. Subagents must not merge or remove worktrees.
7. Close issues with gh issue close <N> --comment "Implemented in <sha>."
Never dispatch two agents to the same crate simultaneously.
Never use --no-verify. Cap at 5 concurrent agents.
