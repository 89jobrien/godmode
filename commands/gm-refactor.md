---
description: "Refactor this scope without changing observable behaviour"
allowed-tools:
  - Bash
  - Read
  - Edit
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

Refactor this scope without changing observable behaviour: $ARGUMENTS
Treat `$ARGUMENTS` as descriptive scope. If empty, ask for the scope and stop.
Follow godmode:refactoring exactly:
1. Run `nu ($env.HOME | path join ".agents" "skills" "refactoring" "helpers" "refactor-gate.nu")` to confirm green baseline.
   If red, stop — fix tests first.
2. State scope: which file(s), what pattern (extract/rename/move/decouple), and why.
3. Make one structural change at a time. Run `cargo nextest run` after each — must stay green.
4. Run `nu ($env.HOME | path join ".agents" "skills" "refactoring" "helpers" "refactor-gate.nu") --after` when done.
5. Run godmode:code-review on your own diff before committing.
Do not combine rename + extract in one step. Do not change behaviour.
