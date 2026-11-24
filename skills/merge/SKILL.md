---
name: "godmode:merge"
description: >
  Use when the user wants to merge a branch, integrate changes, resolve conflicts, create a PR,
  squash-merge a feature branch, or clean up a worktree after work is done. Handles single-branch
  merges, PR creation and merge via gh, squash workflows, worktree cleanup, push rejection recovery,
  and godmode task state sync. Trigger on "merge", "merge this", "open a PR", "squash and merge",
  "clean up worktree", "/merge", or any variant of integrating a branch into main.
allowed-tools: Bash, Read, Glob, Grep
requires: []
next: [changelog, release-notes]
---

# godmode:merge — Branch Integration Agent

General-purpose merge agent. Covers single-branch merges, PR-based workflows, squash mode,
worktree cleanup, and godmode task state sync. For merging multiple parallel agent branches
produced by a wave, use `godmode:wave-integration` instead.

## Reference files

Read the file named in a row before doing the task in its right column. These files are not
loaded automatically.

| File                                                  | Read before                                                                                     |
| ----------------------------------------------------- | ----------------------------------------------------------------------------------------------- |
| `references/mode-rejection-recovery.md`               | recovering from a rejected `git push` (non-fast-forward, stale info, permission denied)         |
| `references/mode-direct-merge.md`                     | merging a feature branch into `main` locally with no PR, or choosing a merge strategy           |
| `references/mode-direct-merge-conflicts-test-push.md` | resolving conflicts during a direct merge, running tests after it, or pushing the merged `main` |
| `references/mode-squash.md`                           | squash-merging a branch, or deciding whether to collapse its commits                            |
| `references/mode-pr.md`                               | opening a PR for the current branch, or merging an open PR via `gh`                             |
| `references/worktree-and-task-sync.md`                | removing a worktree after a merge, or marking a godmode task done                               |

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

Each mode's steps are in the reference file for it, listed above.

## Report

One-line summary when complete:

```text
merged: <branch> -> main | <sha> | [conflicts: N files] [worktree: removed] [task: <id> done]
```

If a PR was created instead of merged, print the PR URL.
If escalating to the user, state exactly what is blocked and what decision is needed.

## Verification

After any merge:

```bash
git log --oneline main..<branch>   # must be empty — branch fully merged
git log --oneline -3               # shows merge commit at tip
```

If `main..<branch>` is non-empty, the merge did not complete. Investigate before reporting done.

## Key rules

- `git branch --show-current` before every commit — stop if on `main` unexpectedly.
- Never `--no-verify`. Let hooks run. Identify and fix failures; do not bypass.
- Never force-push `main`. Feature branches only, explicit user instruction required.
- Never abort a merge without showing `git diff HEAD` first.
- Never rebase branches with merge commits.
- Worktree cleanup only after verifying `git log --oneline main..<branch>` is empty.
- For multi-branch wave integration, use `godmode:wave-integration`.
