---
name: refactor
allowed_tools:
  - Bash
  - Read
  - Edit
  - Glob
  - Grep
max_turns: 30
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

Refactor the specified code without changing observable behaviour.
Follow godmode:refactoring exactly:

1. Run skills/refactoring/helpers/refactor-gate.nu to confirm green baseline.
   If red, stop — fix tests first.
2. State scope: which file(s), what pattern (extract/rename/move/decouple), and why.
3. Make one structural change at a time. Run cargo test after each — must stay green.
4. Run skills/refactoring/helpers/refactor-gate.nu --after when done.
5. Run godmode:code-review on your own diff before committing.
   Do not combine rename + extract in one step. Do not change behaviour.
