---
name: dual-forge-pr-merge
description: Use when a repo is mirrored across GitHub and Gitea and you need to decide where to open or merge a PR. Symptoms - the branch tracks one forge, another forge has a different `main`, or GitHub tooling is available but the active branch lives on Gitea.
---

# Dual-Forge PR Merge

## When to Use

Use this for repositories that exist on more than one forge. The point is to open and merge the PR where the branch and base actually live, not where the first CLI happens to work.

## Commands

```bash
# Inspect remotes and branch tracking
 git remote -v
 git branch -vv

# Compare base branches across forges
 git ls-remote github refs/heads/main
 git ls-remote gitea refs/heads/main

# Check whether the current branch exists on each remote
 branch=$(git branch --show-current)
 git ls-remote github "refs/heads/$branch"
 git ls-remote gitea "refs/heads/$branch"
```

## Rules

- Open the PR on the forge where the branch already exists and the base branch matches your actual integration target.
- If `main` differs across remotes, do not treat them as interchangeable.
- Prefer the forge with branch tracking already configured unless the user explicitly wants the other one.
- Confirm merge target before using `gh` or a forge-specific API.

## Common Failures

| Symptom                                              | Fix                                                                         |
| ---------------------------------------------------- | --------------------------------------------------------------------------- |
| GitHub has no branch but Gitea does                  | Use Gitea for the PR or push the branch to GitHub first                     |
| GitHub `main` is not the same commit as Gitea `main` | Pick the intended forge explicitly; do not merge blindly                    |
| CLI support exists for only one forge                | Use read-only git checks first, then select the correct forge-specific path |
