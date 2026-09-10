---
description: "Review the requested scope or current diff across all severity levels."
allowed-tools:
  - Bash
  - Read
  - Edit
  - Glob
  - Grep
---

## Rules

- Read the full diff before commenting. Never review partial context.
- Group findings as Blocking / Suggestions / Nitpicks.
- Apply ALL severity levels in one pass before committing. Do not commit after
  fixing only blocking issues — one review, one fix commit.
- Run verification after fixes: `cargo clippy --workspace -- -D warnings`,
  `cargo nextest run --workspace`, `cargo fmt --all --check`.
- Never use `--no-verify` on git commits.
- Run `git branch --show-current` before any commit. If on main, STOP.

Run a structured code review on `$ARGUMENTS`; if empty, review the current diff.
Treat non-empty arguments as descriptive file/scope input and validate each referenced path
before passing it as a separately quoted path argument.
Follow godmode:code-review exactly:
1. Run `nu ($env.HOME | path join ".agents" "skills" "code-review" "helpers" "run-review.nu")` (or cargo clippy +
   nextest + fmt manually).
2. Read the full diff before commenting.
3. Group findings as Blocking / Suggestions / Nitpicks.
4. Fix all findings in one pass — do not commit after blocking-only fixes.
5. Re-run the gate after fixes. Use godmode:verification-before-completion before marking done.
