---
name: release-readiness-check
description: Use before cutting a release to verify tags, affected crates, gates, binaries, and target remote. Symptoms - you are about to tag a release, create a GitHub release, or automate publishing and want to catch the obvious failures before mutating history.
---

# Release Readiness Check

## When to Use

Use this right before a release commit, tag, or automated workflow run. It is a preflight that checks whether the repo is clean, the intended version is valid, the tag is unused, and the validation gates are green.

## Commands

```bash
# Verify branch, worktree, and remotes
 git status --short --branch
 git remote -v
 git branch -vv

# Check version/tag availability
 git describe --tags --abbrev=0 --match 'v*'
 git rev-parse "v<version>" 2>/dev/null || true
 git ls-remote --tags <remote> "refs/tags/v<version>"

# Run release gates
 cargo fmt --all --check
 cargo clippy --workspace --all-targets -- -D warnings
 cargo nextest run --workspace
```

## Rules

- Do not cut a release from a dirty worktree unless the release process itself is expected to create the only changes.
- Verify tag availability both locally and on the target remote.
- Confirm the intended target remote before pushing tags.
- Check affected crates before choosing which versions to bump.

## Common Failures

| Symptom                                       | Fix                                                    |
| --------------------------------------------- | ------------------------------------------------------ |
| Release fails because the tag already exists  | Check local and remote tags before committing the bump |
| Release automation mutates the wrong branch   | Verify current branch and remote tracking first        |
| Last-minute clippy failure breaks the release | Run the full release gates before tagging              |
