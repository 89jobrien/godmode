---
name: doc-enrich
allowed-tools:
  - Bash
  - Read
  - Edit
  - Write
  - Glob
  - Grep
max-turns: 30
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

Review, maintain, and sync all project documentation in one pass.

1. Run godmode:doc-review — structured review of existing docs for accuracy,
   completeness, clarity, and navigability. Produce a PASS/FAIL verdict with
   Blocking / Suggestion / Nitpick findings.
2. Run godmode:doc-maintainer — audit CLAUDE.md, README.md, skill descriptions,
   and agent INDEX against actual source code. Fix stale references and undocumented
   features surfaced by doc-review.
3. Run godmode:doc-sync — verify all file paths, CLI flags, crate names, and
   cross-doc consistency. Confirm no drift remains after doc-maintainer fixes.
4. Run godmode:cap — commit all documentation changes.
   Fix Blocking findings from step 1 before proceeding to step 2.
   Do not rewrite documents wholesale — use targeted edits only.
