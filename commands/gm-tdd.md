---
name: tdd
allowed-tools:
- Bash
- Read
- Edit
- Write
- Glob
- Grep
max-turns: 50
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


Implement a feature or fix using strict test-driven development.
Follow godmode:task-driven-development exactly:
1. Write a FAILING test first. Run it — confirm it fails for the right reason.
2. Write the minimum code to make it pass. Run taskit test run — all green.
3. Refactor: taskit check lint --crate-name <crate>, taskit check fmt, taskit test run (still green).
4. Commit: git commit -m "feat(<crate>): <what it does>"
Repeat for each requirement.
Iron law: no production code without a prior failing test. If you wrote code before
the test, delete it and start over.
3-attempt rule: if a test is still failing after 3 attempts, stop and report.
