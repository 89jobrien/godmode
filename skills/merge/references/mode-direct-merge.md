# Direct-merge mode

Read this before merging a single feature branch into main locally with no PR, including choosing a merge strategy. Conflict resolution, tests, and push are in mode-direct-merge-conflicts-test-push.md.

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
