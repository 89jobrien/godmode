---
name: session-end
allowed-tools:
  - Bash
  - Read
  - Write
  - Glob
max-turns: 20
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

Close out a coding session: summarise, reflect, capture learnings, update memory, commit.

1. Run godmode:whatidid — summarise what was accomplished this session from git log
   and task state. Output a one-paragraph session summary before continuing.
2. Run godmode:self-reflect — assess what went well, what went wrong, and what to
   change next session.
3. Run godmode:mistake-tracker — append any new recurring patterns discovered this session
   to the mistake ledger at .ctx/memory-bank/mistakes.md.
4. Run godmode:memory-banking — update active-context.md and progress.md with session
   outcomes. Update other memory bank files only if content changed.
5. Run godmode:session-wrap-commit-push — commit any outstanding clean changes, write
   HANDOFF, and push.
   Run in order. Memory updates must be grounded in what actually happened this session.
