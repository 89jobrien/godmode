---
name: issues
allowed-tools:
- Bash
- Read
- Edit
- Write
- Glob
- Grep
max-turns: 50
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
