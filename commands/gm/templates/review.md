## Rules

- Read the full diff before commenting. Never review partial context.
- Group findings as Blocking / Suggestions / Nitpicks.
- For audit-only requests, report findings without modifying files.
- When remediation is requested, apply ALL severity levels in one pass before
  committing. Do not commit after fixing only blocking issues.
- After fixes, run `taskit check lint`,
  `taskit test run`, and `taskit check fmt --check`.
- After 3 failed fix attempts, write `BLOCKED.md` and stop.
- Never use `--no-verify` on git commits.
- Run `git branch --show-current` before any commit. If on main, STOP.
