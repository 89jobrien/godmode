---
name: pr
allowed-tools:
- Bash
- Read
- Edit
- Write
- Glob
- Grep
max-turns: 30
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


Author and open a pull request from the current branch.
1. Run godmode:code-review — review the branch diff for bugs, style, and correctness.
   Fix any blocking findings before continuing.
2. Run godmode:doublecheck — independent second pass to catch anything missed.
   Fix any blocking findings.
3. Run godmode:pr-author — compose the PR description from branch diff, task context,
   and commit history. Produce gh pr create command ready to run.
4. Open the PR with gh pr create. Report the PR URL.
5. Run godmode:merge — once CI is green and review is approved, merge the PR.
Do not open the PR until code-review and doublecheck both pass clean.
Pause after step 3 for user confirmation before opening the PR.
