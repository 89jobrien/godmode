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
