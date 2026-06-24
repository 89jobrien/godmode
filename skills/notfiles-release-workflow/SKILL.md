---
name: notfiles-release-workflow
description: Use when applying the global Rust release-workflow pattern to the notfiles repo. Symptoms - you need the repo-specific release file, affected-crate exclusions, binary list, or GitHub-vs-Gitea cautions for notfiles.
---

# Notfiles Release Workflow

## When to Use

Use this after `rust-release-workflow-author` when the target repo is `notfiles`. This skill only carries the repo-specific deltas that are not worth baking into the general Rust release skill.

## Repo-Specific Facts

- Release workflow path: `.github/workflows/release.yml`
- Current release binaries to package:
  - `notfiles`
  - `nothooks`
  - `notstrap`
  - `notgraph`
- Test-only workspace member to exclude from version bumps:
  - `integration`
- Validation gates expected before tag push:
  - `cargo fmt --all --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo nextest run --workspace`

## Commands

```bash
# Read the repo workflows and package list
# Use Glob tool: glob '.github/**/*'
sed -n '1,260p' .github/workflows/ci.yml
sed -n '1,260p' .github/workflows/nightly.yml
cargo metadata --no-deps --format-version 1
rg -n '^version\s*=|^name\s*=' Cargo.toml crates/*/Cargo.toml

# Review the repo-specific release file after authoring
sed -n '1,320p' .github/workflows/release.yml
```

## Rules

- Prefer `rust-release-workflow-author` for the general workflow shape; keep this skill focused on `notfiles` specifics.
- Exclude the `integration` package from release bumps.
- Package only the four current release binaries unless the repo adds or removes top-level CLIs.
- Treat `notcore` and other shared workspace-crate changes as release-impacting for downstream consumers.
- Be explicit about target forge: this repo may have both GitHub and Gitea remotes, and they may not share the same `main`.

## Common Failures

| Symptom                                                       | Fix                                                                                   |
| ------------------------------------------------------------- | ------------------------------------------------------------------------------------- |
| A release tries to bump the `integration` package             | Filter test-only workspace members out of the affected crate set                      |
| GitHub release flow is authored against the wrong base branch | Check GitHub and Gitea `main` separately before finalizing the workflow               |
| Asset packaging misses a CLI                                  | Verify the workflow still packages `notfiles`, `nothooks`, `notstrap`, and `notgraph` |
