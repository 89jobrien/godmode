---
description: issues
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

Triage, work, and close GitHub issues end-to-end.

1. Run godmode:issue-triage — fetch open issues, classify by type (bug/feature/chore),
   complexity (S/M/L), and priority (P1-P3). Present the triage table and wait for
   user confirmation on which issues to work.
2. Run godmode:tackle-issues — dispatch confirmed issues to parallel worktrees,
   one agent per issue slot (cap 5). Each agent implements, tests, and commits.
3. Run godmode:todo-issue-sync — audit inline TODOs added or resolved during
   implementation; sync status back to GitHub issues.
4. Run godmode:cap — push all branches and open PRs for completed issues.
   Pause after step 1 for user confirmation on the triage list before dispatching.
   Do not work P3 issues without explicit approval.
