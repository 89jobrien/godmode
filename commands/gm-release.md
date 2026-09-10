---
description: "Run workspace release readiness, impact, versioning, documentation, and publication steps."
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

Workspace release pipeline: readiness check, impact analysis, version bumps, docs, commit, push.
1. Run godmode:release-readiness-check — verify tags, gates, affected crates, and target remote.
   Abort if any check fails; surface what needs fixing.
2. Run godmode:workspace-release-impact — identify which crates need version bumps from
   the current change set. Report the impact list and wait for user confirmation.
3. Run godmode:workspace-bump-commit — apply cargo set-version to affected crates,
   stage manifests and lockfile, create release commit.
4. Run godmode:changelog — update CHANGELOG.md.
5. Run godmode:release-notes — produce human-facing release notes.
6. Run godmode:cap — push the release commit.
Never bump versions without a passing readiness check. Only bump crates identified
by workspace-release-impact. Pause after step 2 for user confirmation before step 3.
