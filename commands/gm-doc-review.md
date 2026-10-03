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
