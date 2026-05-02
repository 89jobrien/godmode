---
name: "godmode:tackle-issues"
description: >
  Use when the user wants to work on GitHub issues in parallel — "tackle issues", "fix these
  issues", "work on #7 #8 #9", "dispatch agents for open issues". Fetches issues, groups them
  into independent units, and dispatches one godmode-crate-agent per issue (capped at 5 concurrent).
  Triggers on issue numbers, "tackle", "fix issues", or "dispatch for issues".
---

# Tackle Issues

Fetch GitHub issues, group into independent units, dispatch parallel subagents.

## Step 1: Resolve issues

If the user named specific issue numbers, use those. Otherwise fetch open issues:

```bash
gh issue list --repo <owner>/<repo> --state open --limit 20 \
  --json number,title,body,labels
```

Read each issue body. Extract:

- **Crate scope** — look for `crate:`, `**Crate**:`, or infer from file paths mentioned
- **Files to create/modify** — listed in the issue body
- **Dependencies between issues** — note if issue B says "after #X" or depends on a type
  from another issue

## Step 2: Independence check

Two issues are **independent** if:

- They target different crates, OR
- They target the same crate but touch non-overlapping files

Two issues are **dependent** if:

- One introduces a type or trait consumed by the other
- One says "after #N" or "depends on #N"

Group independent issues into parallel slots. Chain dependent issues sequentially within
a slot. Cap at **5 concurrent slots**.

Present the grouping for confirmation before dispatching (example):

```
Slot 1 (independent): #7 session-start hook
Slot 2 (independent): #8 pre-commit hook
Slot 3 (independent): #9 task-done-sync hook
```

Wait for user go/no-go.

## Step 3: Prepare worktrees

Resolve the repo root first:

```bash
REPO_ROOT=$(git rev-parse --show-toplevel)
```

Ensure `.worktrees/` is gitignored (add if absent). Use the Grep tool to check, then append
if missing:

```bash
# Check: Grep for '.worktrees/' in $REPO_ROOT/.gitignore
# If not found: echo '.worktrees/' >> "$REPO_ROOT/.gitignore"
```

For each slot, create a worktree branched from the latest main:

```bash
git -C "$REPO_ROOT" fetch origin main
git -C "$REPO_ROOT" worktree add "$REPO_ROOT/.worktrees/issue-<N>" -b issue/<N>
```

One worktree per slot. Never reuse a worktree across slots. All subsequent git and cargo
commands use absolute paths anchored to `$REPO_ROOT`.

## Step 4: Dispatch agents

Spawn one `godmode-crate-agent` per slot. Each agent prompt must be self-contained:

```
You are implementing GitHub issue #<N>: <title>

Worktree absolute path: <REPO_ROOT>/.worktrees/issue-<N>
Branch: issue/<N>

Issue body:
<full issue body>

Workflow:
1. Run: git -C <REPO_ROOT>/.worktrees/issue-<N> branch --show-current
   Verify output = issue/<N>. Stop immediately if not.
2. Read all files listed in the issue body before writing anything.
   Use absolute paths — do NOT use cd.
3. For each file to create/modify:
   a. Write a FAILING test if applicable.
      Run: cargo nextest run --workspace --manifest-path <REPO_ROOT>/Cargo.toml
      Confirm FAIL.
   b. Implement minimum code to pass.
      Run: cargo nextest run --workspace --manifest-path <REPO_ROOT>/Cargo.toml
      All green.
   c. Run: cargo clippy --workspace --manifest-path <REPO_ROOT>/Cargo.toml -- -D warnings
      Zero warnings.
4. Commit:
   git -C <REPO_ROOT>/.worktrees/issue-<N> add -A
   git -C <REPO_ROOT>/.worktrees/issue-<N> commit -m "feat: <summary> fixes #<N>"
5. If a matching godmode task exists (check .ctx/GODMODE.tasks.yaml):
   godmode task done <task-id> --commit <sha>
   If no matching task exists, skip this step.
6. Final check:
   cargo nextest run --workspace --manifest-path <REPO_ROOT>/Cargo.toml
   cargo clippy --workspace --manifest-path <REPO_ROOT>/Cargo.toml -- -D warnings
7. Report: files created, tests added, commit SHA, any blockers.

If stuck after 3 attempts on any item: write BLOCKED.md at <REPO_ROOT>/.worktrees/issue-<N>/BLOCKED.md. Stop.
Do NOT modify files outside <REPO_ROOT>/.worktrees/issue-<N>.
```

Pass explicit tools — agents do NOT inherit permissions:

```
allowedTools: Read, Write, Edit, Bash, Grep, Glob
```

## Step 5: Integrate results

After all agents report:

1. Verify each agent committed:

```bash
git -C "$REPO_ROOT/.worktrees/issue-<N>" log --oneline -3
```

Empty log = agent did not finish. Escalate to user, do not proceed with that slot.

2. Merge each branch into main sequentially (never octopus):

```bash
git -C "$REPO_ROOT" checkout main
git -C "$REPO_ROOT" merge --no-ff issue/<N> -m "merge: issue #<N> <title>"
```

Resolve any conflicts before moving to the next branch.

3. Run full workspace suite from main:

```bash
cargo nextest run --workspace --manifest-path "$REPO_ROOT/Cargo.toml"
cargo clippy --workspace --manifest-path "$REPO_ROOT/Cargo.toml" -- -D warnings
```

4. Remove merged worktrees:

```bash
git -C "$REPO_ROOT" worktree remove "$REPO_ROOT/.worktrees/issue-<N>"
git -C "$REPO_ROOT" branch -d issue/<N>
```

5. Confirm merge landed on main before closing:

```bash
git -C "$REPO_ROOT" log --oneline main | head -5   # verify merge commit present
```

6. Close issues:

```bash
gh issue close <N> --repo <owner>/<repo> --comment "Implemented in <commit-sha>."
```

7. Sync to godmode: mark any running godmode tasks done per issue:

```bash
godmode task done <task-id> --commit <merge-sha>
```

Skip if no matching task exists in `.ctx/GODMODE.tasks.yaml`.

## Guardrails

- Never dispatch two agents to the same crate simultaneously.
- Each agent must verify `git branch --show-current` before every commit.
- Cap at 5 concurrent agents — queue the rest.
- Never use `--no-verify` on commits.
- A slot with `BLOCKED.md` is escalated to the user, not retried automatically.
- Do not close an issue until its merge is confirmed in `git log main`.
