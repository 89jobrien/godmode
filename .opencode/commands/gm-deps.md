---
description: Audit and update workspace dependencies.
subtask: false
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


Audit and update workspace dependencies.
1. Run godmode:dep-audit — identify outdated, yanked, or vulnerable dependencies.
   Present findings grouped by severity: Critical (yanked/vuln) → Outdated → Minor.
   Wait for user confirmation on which deps to update.
2. Run godmode:dep-bump — apply approved updates. Review changelogs for breaking changes
   before each bump. Run taskit test run after each bump to catch regressions.
3. Run godmode:cap — commit and push with message: chore(deps): bump <names>.
Never bump a dependency without reviewing the changelog first.
Abort if any test fails after a bump — report which dep caused the failure.
