## Rules

- Read the full diff before commenting. Never review partial context.
- Group findings as Blocking / Suggestions / Nitpicks.
- Apply ALL severity levels in one pass before committing. Do not commit after
  fixing only blocking issues — one review, one fix commit.
- Run verification after fixes: `cargo clippy --workspace -- -D warnings`,
  `cargo nextest run --workspace`, `cargo fmt --all --check`.
- Never use `--no-verify` on git commits.
- Run `git branch --show-current` before any commit. If on main, STOP.
