---
name: doc-review
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

Review documentation for accuracy, completeness, clarity, and navigability before publishing.
Follow godmode:doc-review exactly:

1. Accuracy: run every documented command, check every path, flag, type, and module name
   against actual source. Flag any unverifiable claim as Blocking.
2. Completeness: confirm install/setup, CLI reference, and key concepts are fully covered.
   Flag gaps as Suggestion.
3. Clarity: confirm entry point is obvious, terms are defined before use, examples are
   concrete (real commands, real output). Flag confusion as Suggestion or Nitpick.
4. Navigability: check headings, TOC for long docs (>200 lines), and cross-reference links.
   Flag broken links as Blocking, missing TOC as Suggestion.
   Report using the format:

## Doc Review: <filename>

### Blocking / Suggestion / Nitpick

**Verdict**: PASS | FAIL
PASS = no Blocking findings. On FAIL, hand back to godmode:doc-writer to fix.
On PASS, hand off to godmode:cap to commit.
