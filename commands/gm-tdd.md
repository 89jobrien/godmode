---
description: "Implement this feature or fix using strict test-driven development"
allowed-tools:
  - Bash
  - Read
  - Edit
  - Write
  - Glob
  - Grep
---

## Rules

- Always run `git branch --show-current` before any commit. If on main, STOP.
- Never use `--no-verify` on git commits.
- Derive a concrete conventional commit type, scope, and summary from the diff; never
  commit a message containing placeholders.
- Cargo gates before committing: `cargo fmt --all`, `cargo clippy --workspace -- -D warnings`,
  `cargo nextest run --workspace`.
- 3-attempt rule: if a test or fix fails 3 times, stop and report the root cause.
  Do not continue patching.
- Run `cargo fmt --all` then re-stage before committing — the PostToolUse hook
  runs fmt automatically but does not stage.
- Commits are signed via SSH key through 1Password. If signing fails, tell the
  user to unlock 1Password — do not change git config.
- Scratch files go in `.ctx/godmode/_WORKING_DIR/`.

Implement this feature or fix using strict test-driven development: $ARGUMENTS
Treat `$ARGUMENTS` as descriptive text, never as shell syntax. If it is empty, ask for
the required feature or fix and stop.
Follow godmode:task-driven-development exactly:
1. Write a FAILING test first. Run it — confirm it fails for the right reason.
2. Write the minimum code to make it pass. Run `cargo nextest run` — all green.
3. Refactor: resolve the actual package name from workspace metadata, then run
   `cargo clippy -p "$package" -- -D warnings`, `cargo fmt`, and `cargo nextest run`.
4. Derive a concrete conventional commit message from the diff. Never leave angle-bracket
   placeholders in commands or commit messages.
Repeat for each requirement.
Iron law: no production code without a prior failing test. If you wrote code before
the test, delete it and start over.
3-attempt rule: if a test is still failing after 3 attempts, stop and report.
