---
name: tdd
allowed_tools:
  - Bash
  - Read
  - Edit
  - Write
  - Glob
  - Grep
max_turns: 50
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

Implement a feature or fix using strict test-driven development.
Follow godmode:task-driven-development exactly:

1. Write a FAILING test first. Run it — confirm it fails for the right reason.
2. Write the minimum code to make it pass. Run cargo nextest — all green.
3. Refactor: cargo clippy -p <crate> -- -D warnings, cargo fmt, cargo nextest (still green).
4. Commit: git commit -m "feat(<crate>): <what it does>"
   Repeat for each requirement.
   Iron law: no production code without a prior failing test. If you wrote code before
   the test, delete it and start over.
   3-attempt rule: if a test is still failing after 3 attempts, stop and report.
