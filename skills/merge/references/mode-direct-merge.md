# Direct-merge mode

Read this before merging a single feature branch into `main` locally with no PR, including choosing a merge strategy and resolving conflicts.

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
taskit test run
```

Fix failures (up to 3 attempts) before pushing. Escalate if not resolved.

### 6. Push

```bash
git push
```

If no upstream: `git push -u origin $(git branch --show-current)`
Never use `--no-verify`.
