---
description: 'Pre-release code quality pass: refactor, test discipline, quality score, readiness check, changelog, commit.'
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


Pre-release code quality pass: refactor, test discipline, quality score, readiness check, changelog, commit.
1. Run godmode:refactoring — identify and apply structural improvements without
   changing observable behaviour. Run taskit test run after each change — must stay green.
2. Run godmode:testing-philosophy — review test coverage and test types. Identify
   gaps: missing unit tests, integration tests that should be property tests, etc.
   Add missing tests for any code touched in step 1.
3. Run godmode:rustqual — check quality score. Address any HIGH findings.
4. Run godmode:release-readiness-check — verify tags, gates, affected crates, and
   CI status. Abort if any check fails.
5. Run godmode:changelog — update CHANGELOG.md from commits since last tag.
6. Run godmode:cap — commit the polished state.
Do not proceed past step 4 if readiness check fails — surface what needs fixing.
This is a quality pass, not a feature pass — scope is strictly improvement, not new behaviour.
