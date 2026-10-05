---
name: cap
allowed-tools:
- Bash
- Read
- Edit
- Write
- Glob
- Grep
max-turns: 15
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


Commit and Push. Run the cap workflow: validate via the git hooks (format, clippy, tests), stage all changes,
derive a conventional commit message from the diff, commit, and push.
Guard: if on main branch, stop and report — do not commit.
Use skills/cap/helpers/cap.nu or follow godmode:cap exactly.
