---
name: fresh-branch
allowed-tools:
- Bash
- Read
- Edit
- Write
- Glob
- Grep
max-turns: 25
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


Create a fresh branch from origin/main containing only the intended changes.

Arguments: $ARGUMENTS (branch name suffix or description, e.g. "fix/my-fix")

1. Run `git branch --show-current` — note the current branch.
2. Run `git stash` to save any uncommitted changes.
3. Run `git fetch origin main` then `git checkout -b $ARGUMENTS origin/main`.
4. If there are specific commits to include, cherry-pick them by SHA.
   Otherwise, re-implement the fix cleanly from scratch on this branch.
5. Run `git log --oneline origin/main..HEAD` — verify ONLY the intended commits are present.
   If unrelated commits appear, STOP and report to the user before proceeding.
6. Run `taskit check fmt --check`, `taskit check lint`,
   and `taskit test run`. Fix any failures before pushing.
7. Push the branch and report the branch name and commit SHAs included.
