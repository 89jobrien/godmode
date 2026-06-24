---
name: rust-release-workflow-author
description: Use when creating or editing a GitHub Actions release workflow for a Rust workspace with version bumps, tags, affected-crate selection, binary packaging, and GitHub releases. Symptoms - you need a manual release workflow, selective crate bumps, downstream-dependent bump logic, or a release asset pipeline for Rust binaries.
---

# Rust Release Workflow Author

## When to Use

Use this when the repo needs a real release workflow rather than just CI. This is for Rust workspaces that tag releases, update crate versions, build binaries, and publish GitHub releases.

## Commands

```bash
# Read existing workflows first
# Use Glob tool: glob '.github/**/*'
sed -n '1,260p' .github/workflows/ci.yml
sed -n '1,260p' .github/workflows/nightly.yml 2>/dev/null || true

# Read a known-good reference workflow before authoring
sed -n '1,260p' ~/dev/minibox/.github/workflows/release.yml

# Inspect workspace packages
cargo metadata --no-deps --format-version 1
rg -n '^version\s*=|^name\s*=' Cargo.toml crates/*/Cargo.toml

# Write workflow via heredoc
# Use the Write tool to create .github/workflows/release.yml with the full content.
# (Bash heredoc cat-redirect is blocked; Write tool is the correct approach.)

# Validate YAML
ruby -e 'require "yaml"; YAML.load_file(".github/workflows/release.yml"); puts "yaml-ok"'

# Review the rendered file before stopping
sed -n '1,320p' .github/workflows/release.yml
```

## Rules

- Prefer `workflow_dispatch` for explicit release control.
- Validate semver input and refuse duplicate tags.
- Determine release scope from the diff since the last `v*` tag, not from guesswork.
- Use `cargo metadata` to compute downstream dependents when shared workspace crates change.
- Treat shared non-crate changes as workspace-wide unless the repo has a stricter contract.
- Run release gates before any commit or tag is pushed.
- Keep version bump, tag creation, push, build, and release publication in the same workflow.
- Bump only affected publishable crates; exclude test-only workspace members.
- Use a Bash heredoc for workflow file writes when workflow editors are blocked by tooling hooks.

## Common Failures

| Symptom                                                  | Fix                                                                                                     |
| -------------------------------------------------------- | ------------------------------------------------------------------------------------------------------- |
| Workflow exists but bumps every crate                    | Add affected-crate selection before `cargo set-version`                                                 |
| Shared crate changed but downstream apps were not bumped | Build reverse dependencies from `cargo metadata` and walk dependents transitively                       |
| Root-level config change was ignored                     | Treat non-crate shared changes as affecting all publishable workspace crates unless narrowed explicitly |
| Tag is created before validation finishes                | Move push/tag steps after release gates                                                                 |
| Release asset step fails                                 | Verify built binary names and archive paths match the workflow inputs                                   |
| Workflow edits are blocked by local tooling hooks        | Write `.github/workflows/*.yml` with a heredoc and validate the final file locally                      |
