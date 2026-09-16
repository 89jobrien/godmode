---
description: deps
subtask: false
---

## Rules

- Always run `git branch --show-current` before any commit. If on main, STOP.
- Never use `--no-verify` on git commits.
- Conventional commits: `feat(<crate>):`, `fix(<crate>):`, `refactor(<crate>):`.
- Cargo gates before committing: `cargo fmt --all`, `cargo clippy --workspace -- -D warnings`,
  `cargo nextest run --workspace`.
- 3-attempt rule: if a test or fix fails 3 times, stop and report the root cause.
  Do not continue patching.
- Run `cargo fmt --all` then re-stage before committing — the PostToolUse hook
  runs fmt automatically but does not stage.
- Commits are signed via SSH key through 1Password. If signing fails, tell the
  user to unlock 1Password — do not change git config.
- Scratch files go in `.ctx/_WORKING_DIR/`.

Audit and update workspace dependencies.

1. Run godmode:dep-audit — identify outdated, yanked, or vulnerable dependencies.
   Present findings grouped by severity: Critical (yanked/vuln) → Outdated → Minor.
   Wait for user confirmation on which deps to update.
2. Run godmode:dep-bump — apply approved updates. Review changelogs for breaking changes
   before each bump. Run cargo test after each bump to catch regressions.
3. Run godmode:cap — commit and push with message: chore(deps): bump <names>.
   Never bump a dependency without reviewing the changelog first.
   Abort if any test fails after a bump — report which dep caused the failure.
