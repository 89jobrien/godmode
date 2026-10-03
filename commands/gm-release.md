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
- Run quality gates through `taskit`, never raw `cargo`.
- Do NOT run gates by hand before committing or pushing. The global git hooks run format
  check, clippy on affected crates, tests, and the secret scan. Fix what a hook reports
  instead of re-running it. One exception: a format failure is detected but not fixed, so
  run `taskit check fmt` and `git add` the result.
- 3-attempt rule: after 3 failed attempts, write `BLOCKED.md` with the attempts
  and root cause, then stop. Do not continue patching.
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
