---
description: audit
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

Full repo health audit: quality, dead code, dependencies, recurring mistakes, and backlog gaps.

1. Run godmode:health-score — overall quality signal and trend.
2. Run godmode:dead-code — identify unused exports, types, and functions.
3. Run godmode:dep-audit — flag outdated, yanked, or vulnerable dependencies.
4. Run godmode:mistake-tracker — surface recurring error patterns from session traces and git history.
5. Run godmode:repo-gap-backlog — identify missing issues, docs, or spec gaps.
   Synthesise all findings into a prioritised list: Critical → High → Medium → Low.
   Suggest which gm- command addresses each category. Read-only — do not fix anything here.
