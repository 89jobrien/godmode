---
description: "Execute an ingested task graph using test-driven development."
allowed-tools:
  - Bash
  - Read
  - Edit
  - Write
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

Implement the ingested godmode task graph: $ARGUMENTS
Treat `$ARGUMENTS` as an optional task focus, never as shell syntax.
Invoke godmode:task-management to inspect the graph, then
godmode:task-driven-development to execute each runnable task in dependency order.
If the graph is absent or contains no pending work, stop and tell the user to run
`/gm:ingest` first. Preserve strict failing-test, minimal-implementation, refactor, and
verification phases. Never skip blocked dependencies or the three-attempt rule.
