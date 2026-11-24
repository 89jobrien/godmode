# Direct-merge mode: conflicts, tests, and push

Read this when resolving conflicts during a direct merge, running tests after the merge, or pushing the merged main.

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
