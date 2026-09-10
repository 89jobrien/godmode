---
description: "Create a clean guarded branch from the latest origin/main."
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

Create a fresh branch from origin/main containing only the intended changes.

Arguments: $ARGUMENTS (branch name suffix or description, e.g. "fix/my-fix")
Require exactly one non-empty branch name. Derive a branch name containing only
ASCII letters, digits, `.`, `_`, `/`, and `-`; never interpolate raw `$ARGUMENTS`
into a shell command. Validate the derived name with
`git check-ref-format --branch "$branch_name"`.

1. Run `git branch --show-current` — note the current branch.
2. Run `git stash` to save any uncommitted changes.
3. Run `git fetch origin main` then `git checkout -b "$branch_name" origin/main`.
4. If there are specific commits to include, cherry-pick them by SHA.
   Otherwise, re-implement the fix cleanly from scratch on this branch.
5. Run `git log --oneline origin/main..HEAD` — verify ONLY the intended commits are present.
   If unrelated commits appear, STOP and report to the user before proceeding.
6. Run `cargo fmt --check --all`, `cargo clippy --workspace -- -D warnings`,
   and `cargo nextest run --workspace`. Fix any failures before pushing.
7. Push the branch and report the branch name and commit SHAs included.
