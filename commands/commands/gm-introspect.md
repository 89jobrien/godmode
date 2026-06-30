---
name: introspect
allowed_tools:
  - Bash
  - Read
  - Edit
  - Write
  - Glob
  - Grep
max_turns: 40
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

Audit all godmode skills, agents, and plugin files for internal consistency.
Follow godmode:introspection exactly:

1. Run skills/introspection/helpers/audit.nu — fix any broken references or missing index entries.
2. Cross-reference every godmode subcommand call against CLAUDE.md CLI reference.
3. Check tool hygiene: flag cat/grep/find/cd&&git/--no-verify/gh run watch anti-patterns.
4. Check cross-skill consistency: merge strategy, branch guard, concurrency cap (5), BLOCKED.md trigger (3).
5. Fix all findings in one pass. Commit with "fix(skills): introspection corrections — <summary>".
