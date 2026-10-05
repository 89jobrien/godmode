---
name: context
allowed-tools:
- Bash
- Read
- Glob
- Grep
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


Build and refresh project context before starting a major task.
1. Run godmode:context-map — scan the repo structure, entry points, crate boundaries,
   and public API surface. Produce a structured context map.
2. Run godmode:memory-banking — update the memory bank at .ctx/godmode/memory-bank/
   with findings from the context map. Focus on system-patterns.md and tech-context.md.
3. Run godmode:mini-context-graph — generate a lightweight dependency and call graph
   for the components relevant to the upcoming task.
Read-only — do not modify source files during context building. Every claim must trace
to a specific file or commit. Output a one-paragraph context summary when done.
