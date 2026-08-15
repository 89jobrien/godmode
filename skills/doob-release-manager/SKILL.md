---
name: doob-release-manager
description: Manages the doob release pipeline — bumps Cargo.toml version, runs ci.sh quality gates, creates the signed git tag, pushes, and verifies GitHub Actions release artifacts. Use when cutting a new doob release, or when quartermaster delegates a doob-specific release.
---

# doob Release Manager

Orchestrates the doob release process from version bump to artifact verification.

## Pre-Flight Checks

```bash
git status                  # must be clean
git branch --show-current   # must be "main" — releases are cut from main
git log --oneline -5
```

## Release Steps

1. **Determine version** — read `Cargo.toml`, confirm bump type (patch/minor/major) with user,
   compute new version.
2. **Bump version** — edit `Cargo.toml` `version = "X.Y.Z"` field only, no other changes.
3. **Run quality gates** — `./ci.sh` must pass fully. If any check fails, stop and report; do not
   proceed to tag.
4. **Commit version bump** — `git add Cargo.toml Cargo.lock && git commit -m "chore: bump version to vX.Y.Z"`
5. **Create signed tag** — `git tag -s "vX.Y.Z" -m "Release vX.Y.Z"`. If 1Password agent error,
   stop and instruct user to unlock 1Password.
6. **Push tag** — confirm with user before pushing, then `git push origin main` and
   `git push origin "vX.Y.Z"`.
7. **Verify release** — `gh run list --limit 5` and `gh release list --limit 3`; confirm the
   release workflow triggered and artifacts appear with SHA256 checksums.

## Rollback

If anything goes wrong after tagging but before push:

```bash
git tag -d "vX.Y.Z"
git reset --soft HEAD~1
```

## Rules

- Never skip `ci.sh` — one bad release binary breaks all users.
- Never force-push tags.
- Always confirm before `git push origin <tag>`.
