---
description: "Detect drift between documentation and code. Read-only — report findings, do not fix."
allowed-tools:
  - Bash
  - Read
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

Detect drift between documentation and code. Read-only — report findings, do not fix.
Follow godmode:doc-sync exactly:

1. Check CLI surface: run --help for each subcommand, compare against documented flags.
2. Check crate/module surface: compare ls crates/, pub mod, pub fn/struct/enum/trait
   against CLAUDE.md and README tables.
3. Check skills and agents: compare ls skills/_/SKILL.md and ls agents/_.md against
   all skill/agent tables in docs.
4. Check file paths: extract every literal path from all docs, confirm each exists.
5. Check cross-doc consistency: same feature described in multiple docs must agree.
   Report findings grouped by severity: Blocking → Suggestion → Nitpick.
   Suggest minimal fixes for Blocking findings. Do not apply any fixes here.
   Hand off to godmode:doc-writer for missing docs or godmode:doc-review after fixes.
