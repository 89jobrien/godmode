---
name: "godmode:merge"
description: >
  Use when the user wants to merge a branch, integrate changes, resolve conflicts, create a PR,
  squash-merge a feature branch, or clean up a worktree after work is done. Handles single-branch
  merges, PR creation and merge via gh, squash workflows, worktree cleanup, push rejection recovery,
  and godmode task state sync. Trigger on "merge", "merge this", "open a PR", "squash and merge",
  "clean up worktree", "/merge", or any variant of integrating a branch into main.
allowed-tools: Bash, Read, Glob, Grep
---

# godmode:merge — Branch Integration Agent

General-purpose merge agent. Covers single-branch merges, PR-based workflows, squash mode,
worktree cleanup, and godmode task state sync. For merging multiple parallel agent branches
produced by a wave, use `godmode:wave-integration` instead.

## Step 0 — Assess the situation

Before doing anything, gather state:

```bash
git branch --show-current
git status --short
git log --oneline -5
git worktree list
```

Then determine the **mode** (see decision table below). If the user gave explicit instructions,
follow them. If ambiguous, pick the most conservative option and state your choice.

### Mode decision table

| Situation                                | Mode                                                  |
| ---------------------------------------- | ----------------------------------------------------- |
| User said "open a PR" or "create PR"     | **PR-create**                                         |
| PR already open on this branch           | **PR-merge**                                          |
| User said "squash" or "squash and merge" | **Squash**                                            |
| Multiple branches to integrate (wave)    | → defer to `godmode:wave-integration`                 |
| Single feature branch, no PR             | **Direct-merge**                                      |
| Push was rejected                        | **Rejection-recovery** → then re-run appropriate mode |

---

## Mode: Rejection-recovery

```bash
git push 2>&1
```

| Output contains                    | Cause                             | Action                           |
| ---------------------------------- | --------------------------------- | -------------------------------- |
| `non-fast-forward` / `fetch first` | Remote has commits you don't have | Fetch, integrate, retry          |
| `rejected ... (stale info)`        | Force-push needed                 | Ask user — feature branches only |
| `Permission denied`                | Auth failure                      | Check SSH key / 1Password agent  |

```bash
git fetch origin
git log --oneline HEAD..origin/<branch>   # commits on remote you don't have
git log --oneline origin/<branch>..HEAD   # your commits not yet on remote
```

If only remote has new commits: continue to Direct-merge mode.
If both sides diverged: resolve conflicts, then push.
Never force-push to `main`.

---

## Mode: Direct-merge

Integrate the current feature branch into main.

### 1. Fetch and check divergence

```bash
git fetch origin
git log --oneline main..HEAD              # commits to merge
git log --oneline HEAD..origin/main       # new commits on main since branch
```

If `main` has advanced since the branch was cut, review those commits first.

### 2. Choose merge strategy

```bash
git log --oneline --merges main..HEAD     # check for merge commits on branch
```

- Branch has merge commits → use `git merge --no-ff` (never rebase)
- Branch is clean linear history → `git merge --no-ff` is still the default
- User explicitly requested rebase → `git rebase main` (only if no merge commits)

### 3. Merge

```bash
git checkout main
git pull
git merge --no-ff <branch> -m "feat(<scope>): merge <branch>"
```

### 4. Resolve conflicts (if any)

For each conflicted file:

1. Read both sides — understand the intent, do not pick mechanically.
2. Produce a merged result that preserves both intents.
3. `git add <file>`

Complete: `git merge --continue`

If merge becomes too complex, show `git diff HEAD` before considering abort. Never abort without
showing the user what would be lost.

### 5. Test

```bash
cargo nextest run --workspace
```

Fix failures (up to 3 attempts) before pushing. Escalate if not resolved.

### 6. Push

```bash
git push
```

If no upstream: `git push -u origin $(git branch --show-current)`
Never use `--no-verify`.

---

## Mode: Squash

Collapse all branch commits into one before merging, producing a clean linear history on main.

```bash
git checkout main
git pull
git merge --squash <branch>
```

This stages all changes but does not commit. Write a single conventional commit summarizing the
entire branch:

```bash
git commit -m "feat(<scope>): <summary of entire branch>

Squashed from <branch>:
- <key change 1>
- <key change 2>"
```

Then push as normal.

**When to use squash:** Feature branches with noisy WIP commits, branches from orca-strait/parallel
agents where individual commits are scaffolding artifacts, or when the user explicitly requests it.

**When not to use squash:** Branches where individual commits carry meaningful history (e.g., a
multi-phase refactor where each commit represents a reviewable step).

---

## Mode: PR-create

Open a pull request for the current branch.

```bash
git push -u origin $(git branch --show-current)

gh pr create \
  --title "<conventional commit title>" \
  --body "$(cat <<'EOF'
## Summary
- <bullet 1>
- <bullet 2>

## Test plan
- [ ] cargo nextest run --workspace passes
- [ ] clippy clean
EOF
)"
```

Print the PR URL when done. Do not merge — leave that to the user or a subsequent `/merge` call.

---

## Mode: PR-merge

Merge an open PR via gh.

First, check the PR state:

```bash
gh pr view --json number,title,mergeable,mergeStateStatus,reviewDecision
```

If `mergeStateStatus` is not `CLEAN`, report the blockage and stop.

Choose merge style:

- Default: `--merge` (preserves merge commit, matches `--no-ff`)
- Squash requested: `--squash`
- Rebase requested: `--rebase` (only if no merge commits on branch)

```bash
gh pr merge <number> --merge --delete-branch
```

`--delete-branch` removes the remote branch after merge. Pair with worktree cleanup below if
this was a worktree branch.

---

## Worktree cleanup

After a successful merge, check if the source branch was a worktree:

```bash
git worktree list
```

If the merged branch appears as a worktree path, remove it:

```bash
godmode worktree remove <branch>
# or manually:
git worktree remove <path> --force
git branch -d <branch>
```

Only do this after confirming the branch is fully merged:

```bash
git log --oneline main..<branch>   # must be empty
```

---

## Task state sync

If a godmode task is associated with this branch, mark it done:

```bash
godmode task done <task-id> --commit $(git rev-parse HEAD) --notes "merged <branch> into main"
```

If no task ID is known, check `GODMODE.tasks.yaml` for tasks in `running` state that match the
branch name or scope.

---

## Verification

After any merge:

```bash
git log --oneline main..<branch>   # must be empty — branch fully merged
git log --oneline -3               # shows merge commit at tip
```

If `main..<branch>` is non-empty, the merge did not complete. Investigate before reporting done.

---

## Report

One-line summary when complete:

```
merged: <branch> -> main | <sha> | [conflicts: N files] [worktree: removed] [task: <id> done]
```

If a PR was created instead of merged, print the PR URL.
If escalating to the user, state exactly what is blocked and what decision is needed.

---

## Key rules

- `git branch --show-current` before every commit — stop if on `main` unexpectedly.
- Never `--no-verify`. Let hooks run. Identify and fix failures; do not bypass.
- Never force-push `main`. Feature branches only, explicit user instruction required.
- Never abort a merge without showing `git diff HEAD` first.
- Never rebase branches with merge commits.
- Worktree cleanup only after verifying `git log --oneline main..<branch>` is empty.
- For multi-branch wave integration, use `godmode:wave-integration`.
