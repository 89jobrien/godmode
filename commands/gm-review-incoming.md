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
- Cargo gates before committing: `cargo fmt --all`, `cargo clippy --workspace -- -D warnings`,
  `cargo nextest run --workspace`.
- 3-attempt rule: if a test or fix fails 3 times, stop and report the root cause.
  Do not continue patching.
- Run `cargo fmt --all` then re-stage before committing — the PostToolUse hook
  runs fmt automatically but does not stage.
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
