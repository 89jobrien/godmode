---
name: review-incoming
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


Respond to an incoming PR review: triage comments, apply fixes, verify, and push.
1. Run godmode:receiving-review — read all review comments, categorise as
   blocking / non-blocking / nitpick, and produce a response plan.
   Present the plan and wait for user confirmation before making changes.
2. Apply all fixes from the response plan. Address every comment category.
3. Run godmode:verification-before-completion — all gates must be green.
4. Run godmode:cap — commit fixes and push. Reference the PR in the commit message.
Address every review comment — do not silently skip nitpicks.
Do not mark comments resolved until the corresponding fix is committed.
Report a resolution summary when done: comment → fix applied / declined with reason.
