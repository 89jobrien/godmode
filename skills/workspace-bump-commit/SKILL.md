---
name: workspace-bump-commit
description: Use when applying version bumps to selected Rust workspace crates and creating the release commit cleanly. Symptoms - you already know which crates should change, and you need to run `cargo set-version`, stage the right manifests, update the lockfile, and create a release commit without touching unrelated packages.
---

# Workspace Bump Commit

## When to Use

Use this after release impact has already been decided. This is the execution step for applying version changes to the chosen crates and creating the release commit.

## Commands

```bash
# Bump only selected crates
 cargo set-version -p <crate-a> <version>
 cargo set-version -p <crate-b> <version>

# Stage only release-related files
 git add crates/*/Cargo.toml Cargo.lock

# Verify you are on the expected branch
 git branch --show-current
 # If the output is 'main', STOP — do not commit to main directly.

 # Create the release commit and tag
 git commit -m "release: v<version>"
 git tag "v<version>"
```

## Rules

- Only bump crates you have already classified as affected.
- Stage manifests and `Cargo.lock`; avoid unrelated files.
- Create the tag immediately after the release commit so the history is unambiguous.
- Verify the staged diff before committing if the workspace contains unrelated edits.

## Common Failures

| Symptom                                | Fix                                                            |
| -------------------------------------- | -------------------------------------------------------------- |
| Unrelated manifests got bumped         | Use `-p <crate>` per package instead of workspace-wide bumping |
| Release commit includes unrelated work | Stage only manifest and lockfile changes                       |
| Tag points at the wrong commit         | Tag immediately after the release commit, before more edits    |
