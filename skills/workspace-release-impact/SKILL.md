---
name: workspace-release-impact
description: Use when deciding which Rust workspace crates need a version bump from a set of changes. Symptoms - a shared crate changed, downstream crates may also need bumps, or a release process should version only affected packages instead of the entire workspace.
---

# Workspace Release Impact

## When to Use

Use this before a release or when writing release automation for a Rust workspace. It answers: which crates changed directly, which downstream crates are affected transitively, and whether shared root changes should force a broader bump.

## Commands

```bash
# Determine the release diff range
 git describe --tags --abbrev=0 --match 'v*'
 git diff --name-only <last-tag>..HEAD

# Inspect workspace metadata
 cargo metadata --no-deps --format-version 1

# Quick direct-crate scan from changed files
 git diff --name-only <last-tag>..HEAD | awk -F/ '/^crates\/[^/]+\// {print $2}' | sort -u
```

## Rules

- Start from the diff since the last release tag.
- Direct crate changes come from paths under `crates/<name>/`.
- Use `cargo metadata` to compute reverse dependencies and include downstream dependents.
- Treat shared non-crate changes as potentially affecting all publishable crates unless you can prove otherwise.
- Exclude test-only or unpublished workspace members when generating release bumps.

## Common Failures

| Symptom                                                         | Fix                                                               |
| --------------------------------------------------------------- | ----------------------------------------------------------------- |
| Only the changed leaf crate gets bumped                         | Walk reverse dependencies from `cargo metadata`                   |
| Shared crate changed but top-level app versions stayed the same | Include downstream dependents transitively                        |
| Root config changed and no crate was selected                   | Treat shared changes as workspace-wide unless narrowed explicitly |
