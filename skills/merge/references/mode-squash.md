# Squash mode

Read this before squash-merging a branch, or when deciding whether a branch's commits should be collapsed into one before merging to `main`.

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
