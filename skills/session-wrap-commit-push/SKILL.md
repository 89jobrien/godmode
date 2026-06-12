---
name: session-wrap-commit-push
description: Use when ending a coding session and you want to capture handoff state, commit all current changes, and push the branch. Symptoms - you finished a work slice, updated HANDOFF state, and want one repeatable closeout flow instead of running handoff, commit, and git push separately.
---

# Session Wrap Commit Push

## When to Use

Use this at the end of a focused work session after implementation and verification are already done. It captures the durable session state first, then creates a commit from the current diff, then pushes the current branch.

## Commands

```bash
# 1. Refresh the repo handoff state before committing
handoff-detect
sed -n '1,220p' "$(handoff-detect)"
sed -n '1,220p' "$(handoff-detect | sed 's/\.yaml$/.state.yaml/')"

# 2. Stage everything for the closeout commit
git add -A

# 3. Inspect the staged change for commit message generation
git diff --cached --stat
git diff --cached

# 4. Non-blocking secret scan on the staged diff
git diff --cached | redact --level minimal --audit

# 5. Commit with a generated conventional message
git commit -m "<type: short description>"

# 6. Push the current branch to its configured upstream
git push
```

## Commit Message Rules

- Format: `<type>: <short description>`
- Keep it to one line and under 72 characters
- Prefer:
  - `feat:` for new behavior
  - `fix:` for bug fixes
  - `docs:` for handoff-only updates
  - `refactor:` for internal restructuring
  - `chore:` for maintenance changes

## Common Failures

| Symptom                                            | Fix                                                                                             |
| -------------------------------------------------- | ----------------------------------------------------------------------------------------------- |
| `handoff-detect` exits 2                           | Create or migrate the repo handoff first with the handoff workflow                              |
| `redact` reports findings                          | Review the staged diff, but do not block automatically unless the content is actually sensitive |
| `nothing to commit`                                | Stop after the diff check; there is no closeout commit to create                                |
| `1Password: agent returned an error` during commit | Unlock 1Password and retry `git commit`                                                         |
| `git push` says no upstream configured             | Run `git push -u <remote> <branch>` once, then reuse plain `git push`                           |
| push rejected because remote moved                 | Pull/rebase explicitly before retrying; do not force-push by default                            |
