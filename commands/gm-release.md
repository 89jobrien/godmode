---
name: release
allowed-tools:
- Bash
- Read
- Edit
- Write
- Glob
- Grep
max-turns: 40
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


Workspace release pipeline from health assessment through release notes and commit.
Abort when a required readiness or verification step fails; surface what needs fixing.

## Workflow — pipeline: release

1. Run godmode:health-score — Assess repository health and report release blockers.
2. Run godmode:dep-audit — Audit dependency security, licenses, and maintenance risks.
3. Run godmode:code-review — Review release changes across all severity levels in one pass.
4. Run godmode:verification-before-completion — Require every project gate to pass before release work continues.
5. Run godmode:changelog — Update the changelog from verified changes.
6. Run godmode:release-notes — Produce human-facing release notes from the canonical change set.
7. Run godmode:cap — Commit and push the release artifacts only after all prior steps pass.
