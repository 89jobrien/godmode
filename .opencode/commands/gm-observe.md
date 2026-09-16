---
description: observe
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

Post-release observability sweep: query traces, score health, triage surfaced issues.

1. Run godmode:observability-as-infrastructure — query and tail the session trace log
   (.ctx/godmode/traces/trace.jsonl). Surface errors, slow operations, blocked tasks,
   and anomalies since the last release tag.
2. Run godmode:health-score — generate a codebase health scorecard: test count,
   clippy warnings, TODO density, API surface, and coverage trends.
3. Run godmode:issue-triage — for each anomaly or health regression found in steps 1-2,
   create and classify a GitHub issue. Prioritise: P1 for errors in production paths,
   P2 for regressions, P3 for trends worth watching.
   Read-only until step 3. Do not apply fixes here — this workflow surfaces issues for
   the next /gm-issues or /gm-debug-loop cycle.
   Output a one-paragraph health summary when done.
