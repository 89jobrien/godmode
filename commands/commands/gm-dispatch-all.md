---
name: dispatch-all
allowed_tools:
  - Bash
  - Read
  - Write
  - Edit
  - Glob
  - Grep
  - Agent
max_turns: 60
---

## Rules

- Cap parallel subagents at 5 concurrent.
- Each subagent must run `git branch --show-current` before every commit.
  If on main, STOP — do not commit to main directly.
- Worktree subagents MUST merge their branch back and remove the worktree
  before reporting done. An orphaned worktree means the task is incomplete.
- Never use octopus merges — merge sequentially, one branch at a time.
- After each agent completes, verify commits exist: `git log --oneline -3`.
  A HANDOFF with `commits: []` is incomplete.
- If stuck after 3 attempts: write BLOCKED.md and stop.
- Never use `--no-verify` in subagent git operations.
- If `gh` auth fails, log to `.ctx/pending-manual.txt` and continue.
  Do NOT retry auth — tell the user to run `gh auth login`.

Autonomous parallel issue resolution. Fetch all open GitHub issues, dispatch one agent per
independent slot (cap 5), merge sequentially, close issues with commit refs.

Arguments: $ARGUMENTS (optional: issue numbers to target, e.g. "#7 #8 #9"; default: all open)

## Step 1: Fetch issues

Run: gh issue list --state open --limit 50 --json number,title,body,labels

If $ARGUMENTS specifies issue numbers, filter to those. Otherwise use all open issues.

## Step 2: Independence analysis

Two issues are independent if they target different crates OR touch non-overlapping files.
Two issues are dependent if one introduces a type/trait consumed by the other, or says
"after #N" / "depends on #N".

Group into parallel slots (cap 5). Chain dependent issues sequentially within a slot.
Present the grouping and wait for user go/no-go before proceeding.

## Step 3: Worktree setup

```
REPO_ROOT=$(git rev-parse --show-toplevel)
grep -q '.worktrees/' "$REPO_ROOT/.gitignore" || echo '.worktrees/' >> "$REPO_ROOT/.gitignore"
git fetch origin main
# For each slot N:
git worktree add "$REPO_ROOT/.worktrees/slot-N" -b "issue/slot-N"
```

## Step 4: Dispatch agents

Spawn one godmode:gm-crate agent per slot (background). Each agent prompt must include:

- Worktree absolute path
- Branch name (verify with git branch --show-current before every commit)
- Full issue body for each issue in the slot
- TDD workflow: failing test → implement → green → clippy → commit
- Commit message: "feat: <summary> fixes #N"
- If stuck after 3 attempts: write BLOCKED.md and stop
- Never use --no-verify

## Step 5: Integrate results

After each agent completes:

1. Verify commit exists: `git -C "$REPO_ROOT/.worktrees/slot-N" log --oneline -3`
   If empty → escalate to user, skip this slot.
2. Merge sequentially (never octopus):
   `git merge --no-ff issue/slot-N -m "merge: slot-N fixes #..."`
3. Resolve conflicts (expect Cargo.toml workspace dep conflicts — keep both entries).
4. Run full suite: `cargo nextest run --workspace && cargo clippy --workspace -- -D warnings`
5. Remove worktree: `git worktree remove .worktrees/slot-N && git branch -d issue/slot-N`
6. Close issues: `gh issue close N --comment "Implemented in <sha>."`
   If gh auth fails: log to `.ctx/pending-manual.txt` and continue — do NOT retry.

## Step 6: Summary

Report: issues merged, issues blocked, issues pending manual close, total tests passing.
