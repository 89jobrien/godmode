---
description: Run a structured code review on the current diff or specified files.
subtask: false
---
## Rules

- Read the full diff before commenting. Never review partial context.
- Group findings as Blocking / Suggestions / Nitpicks.
- For audit-only requests, report findings without modifying files.
- When remediation is requested, apply ALL severity levels in one pass before
  committing. Do not commit after fixing only blocking issues.
- After fixes, run `cargo clippy --workspace -- -D warnings`,
  `cargo nextest run --workspace`, and `cargo fmt --all --check`.
- After 3 failed fix attempts, write `BLOCKED.md` and stop.
- Never use `--no-verify` on git commits.
- Run `git branch --show-current` before any commit. If on main, STOP.


Run a structured code review on the current diff or specified files.
Follow godmode:code-review exactly:
1. Run skills/code-review/helpers/run-review.nu (or cargo clippy + nextest + fmt manually).
2. Read the full diff before commenting.
3. Group findings as Blocking / Suggestions / Nitpicks.
4. Fix all findings in one pass — do not commit after blocking-only fixes.
5. Re-run the gate after fixes. Use godmode:verification-before-completion before marking done.
