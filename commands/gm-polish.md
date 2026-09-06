---
description: "Pre-release code quality pass: refactor, test discipline, quality score, readiness check, changelog, commit."
allowed-tools:
  - Bash
  - Read
  - Edit
  - Write
  - Glob
  - Grep
---

## Rules

- Always run `git branch --show-current` before any commit. If on main, STOP.
- Never use `--no-verify` on git commits.
- Derive a concrete conventional commit type, scope, and summary from the diff; never
  commit a message containing placeholders.
- Cargo gates before committing: `cargo fmt --all`, `cargo clippy --workspace -- -D warnings`,
  `cargo nextest run --workspace`.
- 3-attempt rule: if a test or fix fails 3 times, stop and report the root cause.
  Do not continue patching.
- Run `cargo fmt --all` then re-stage before committing — the PostToolUse hook
  runs fmt automatically but does not stage.
- Commits are signed via SSH key through 1Password. If signing fails, tell the
  user to unlock 1Password — do not change git config.
- Scratch files go in `.ctx/godmode/_WORKING_DIR/`.

Pre-release code quality pass: refactor, test discipline, quality score, readiness check, changelog, commit.
1. Run godmode:refactoring — identify and apply structural improvements without
   changing observable behaviour. Run `cargo nextest run` after each change — must stay green.
2. Run godmode:testing-philosophy — review test coverage and test types. Identify
   gaps: missing unit tests, integration tests that should be property tests, etc.
   Add missing tests for any code touched in step 1.
3. Run godmode:rustqual — check quality score. Address any HIGH findings.
4. Run godmode:release-readiness-check — verify tags, gates, affected crates, and
   CI status. Abort if any check fails.
5. Run godmode:changelog — update CHANGELOG.md from commits since last tag.
6. Run godmode:cap — commit the polished state.
Do not proceed past step 4 if readiness check fails — surface what needs fixing.
This is a quality pass, not a feature pass — scope is strictly improvement, not new behaviour.
