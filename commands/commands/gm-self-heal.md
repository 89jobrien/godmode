---
name: self-heal
allowed_tools:
  - Bash
  - Read
  - Edit
  - Glob
  - Grep
max_turns: 40
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

Self-healing CI loop. Run cargo clippy + nextest + fmt, diagnose each failure, apply the
minimal fix, re-run, repeat until all three pass clean. Do NOT commit until all green.

Arguments: $ARGUMENTS (optional: specific crate path or --workspace; default: --workspace)

## Loop

Repeat until all gates pass or 10 iterations reached:

### Gate 1: clippy

Run: cargo clippy --workspace --all-targets -- -D warnings
For each warning/error:

- Identify the exact file and line
- Apply the minimal fix (do not refactor surrounding code)
- Re-run clippy immediately after each fix
- Do not move to gate 2 until clippy is clean

### Gate 2: tests

Run: cargo nextest run --workspace
For each failure:

- Read the full failure output — do NOT dismiss as flakiness
- Check environment variables, recent changes, and actual error messages
- Identify root cause before proposing any fix
- Apply fix, re-run the specific failing test first, then full suite
- Do not move to gate 3 until nextest is clean

### Gate 3: fmt

Run: cargo fmt --all --check
If it fails: run `cargo fmt --all` to fix, then re-check.

## Exit conditions

- All 3 gates pass: report a summary of every fix applied and why. Offer to commit.
- 10 iterations reached without passing: write a BLOCKED.md at the repo root listing
  remaining failures and what was attempted. Stop and escalate to user.

## Rules

- Never use --no-verify
- Never skip a gate because "it was passing before"
- Always read actual error output before proposing a fix
- Fix one thing at a time; re-run before fixing the next
