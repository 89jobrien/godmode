---
name: remote-upstream-triage
description: Use when git push, PR creation, or merge flow fails because the current branch has no upstream, the wrong remote is selected, or remote branches have drifted across forges. Symptoms - `git push` says no configured destination, PR tools target the wrong remote, or branch tracking does not match where the work was pushed.
---

# Remote Upstream Triage

## When to Use

Use this when branch tracking or remote selection is the actual problem, not git auth or merge conflicts. The goal is to identify the active remote, set the correct upstream, and avoid pushing or opening a PR against the wrong forge.

## Commands

```bash
# Inspect remotes and branch tracking
 git remote -v
 git branch -vv
 git status --short --branch

# Check where the current branch exists remotely
 git ls-remote <remote> "refs/heads/$(git branch --show-current)"

# Set upstream while pushing the current branch
 git push -u <remote> $(git branch --show-current)

# Verify base branch divergence across remotes
 git rev-parse main
 git rev-parse <remote>/main
```

## Rules

- Identify the remote that already carries the branch before opening a PR.
- Do not assume `origin` is the correct push target.
- Compare base branch SHAs across remotes before using GitHub tooling in dual-forge repos.
- Set upstream once the right remote is confirmed so future plain `git push` works.

## Common Failures

| Symptom                                                    | Fix                                                             |
| ---------------------------------------------------------- | --------------------------------------------------------------- |
| `git push` says no configured push destination             | Push with `-u <remote> <branch>` to set upstream                |
| PR tool targets GitHub but the branch only exists on Gitea | Push or create the PR on the tracked remote instead             |
| `main` differs across remotes                              | Treat the forges as divergent and choose the base intentionally |
