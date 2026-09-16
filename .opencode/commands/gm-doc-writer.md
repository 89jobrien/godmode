---
description: doc-writer
subtask: false
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

Write new documentation grounded in actual code.
Follow godmode:doc-writer exactly:

1. Read all relevant source files before writing — Cargo.toml, entry points, public API,
   CLI --help output, git log. Never invent features or behaviour.
2. Identify the doc type: README, CLAUDE.md, architecture doc, API reference, or skill doc.
3. Write one section at a time. After each section, re-read source to verify every claim.
   Mark any unverifiable claim [UNVERIFIED] rather than guessing.
4. Cross-check before finishing: every flag exists in --help, every path exists,
   every crate name matches Cargo.toml. Resolve all [UNVERIFIED] markers.
5. Hand off to godmode:doc-review when done.
   Output the doc file path when complete.
