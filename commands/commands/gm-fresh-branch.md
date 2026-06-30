---
name: fresh-branch
allowed_tools:
  - Bash
  - Read
  - Edit
  - Write
  - Glob
  - Grep
max_turns: 25
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

Create a fresh branch from origin/main containing only the intended changes.

Arguments: $ARGUMENTS (branch name suffix or description, e.g. "fix/my-fix")

1. Run `git branch --show-current` — note the current branch.
2. Run `git stash` to save any uncommitted changes.
3. Run `git fetch origin main` then `git checkout -b $ARGUMENTS origin/main`.
4. If there are specific commits to include, cherry-pick them by SHA.
   Otherwise, re-implement the fix cleanly from scratch on this branch.
5. Run `git log --oneline origin/main..HEAD` — verify ONLY the intended commits are present.
   If unrelated commits appear, STOP and report to the user before proceeding.
6. Run `cargo fmt --check --all`, `cargo clippy --workspace -- -D warnings`,
   and `cargo nextest run --workspace`. Fix any failures before pushing.
7. Push the branch and report the branch name and commit SHAs included.
