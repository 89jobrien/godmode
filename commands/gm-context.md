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
- Cargo gates before committing: `cargo fmt --all`, `cargo clippy --workspace -- -D warnings`,
  `cargo nextest run --workspace`.
- 3-attempt rule: if a test or fix fails 3 times, stop and report the root cause.
  Do not continue patching.
- Run `cargo fmt --all` then re-stage before committing — the PostToolUse hook
  runs fmt automatically but does not stage.
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
