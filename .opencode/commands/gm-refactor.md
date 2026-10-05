---
description: Refactor the specified code without changing observable behaviour.
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


Refactor the specified code without changing observable behaviour.
Follow godmode:refactoring exactly:
1. Run skills/refactoring/helpers/refactor-gate.nu to confirm green baseline.
   If red, stop — fix tests first.
2. State scope: which file(s), what pattern (extract/rename/move/decouple), and why.
3. Make one structural change at a time. Run taskit test run after each — must stay green.
4. Run skills/refactoring/helpers/refactor-gate.nu --after when done.
5. Run godmode:code-review on your own diff before committing.
Do not combine rename + extract in one step. Do not change behaviour.
