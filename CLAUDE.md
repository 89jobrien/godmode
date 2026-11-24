# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this
repository.

Read local memory: @`.claude.local.md`

## Build & Test

All gates go through `taskit` — never invoke `cargo` directly.

## Rust Conventions

- Gates go through `taskit` — never invoke `cargo` directly.
- Do NOT run gates by hand before committing or pushing. The global git hooks run format
  check, clippy on affected crates, tests, and the secret scan. Fix what a hook reports.
- A format failure is the one case the hook detects but does not fix: run `taskit check fmt`
  then `git add` the reformatted files.
- Clippy runs with `-D warnings`; fix warnings proactively rather than suppressing them.
- Do not investigate rust-analyzer or IDE diagnostics unless explicitly asked — they are often
  stale.

## Git Operations

- NEVER use `--no-verify` on git commits. Always let pre-commit hooks run.
- Before claiming a branch is merged, verify with `git log --oneline main..branch` — empty
  output means fully merged.
- Never drop git stashes without showing the diff and getting explicit confirmation.
- Scope staged changes precisely to the current task. Do not stage unrelated changes.

## Sentinel Reviews

Apply ALL severity levels (blocking, suggestion, nitpick) in one pass before committing.
Do not commit after fixing only blocking issues and leave suggestions for a follow-up — that
creates noisy multi-pass fix histories. One sentinel run, one fix commit.

## Output Style

- No superlatives in generated output. Do not use "impressive", "beautifully", "remarkable",
  "industrial-scale", or similar inflated language. State facts plainly.
- No emojis unless explicitly requested.
- No sycophantic openers ("Great question!", "Absolutely!").
- Act first, explain later. When a task is clear, do it — don't narrate the approach first.

## Detailed context

Read the file named in a row before doing the task in its right column. These files are not loaded automatically.

| File                                               | Read before                                                          |
| -------------------------------------------------- | -------------------------------------------------------------------- |
| `docs/agent-context/AGENTS-build-gates.md`         | running any build, test, lint, format, coverage, or audit gate       |
| `docs/agent-context/AGENTS-install-cli.md`         | rebuilding or installing the godmode binary                          |
| `docs/agent-context/AGENTS-architecture.md`        | changing code in either crate or locating where logic lives          |
| `docs/agent-context/AGENTS-state-file.md`          | reading or writing the task state file                               |
| `docs/agent-context/AGENTS-agent-scratch.md`       | writing drafts, plans, or BLOCKED.md files during a session          |
| `docs/agent-context/AGENTS-trace-events.md`        | changing session tracing or trace output                             |
| `docs/agent-context/AGENTS-integration-pattern.md` | adding or changing an integration with an external tool              |
| `docs/agent-context/AGENTS-json-flag.md`           | adding or changing a subcommand output or its --json support         |
| `docs/agent-context/AGENTS-plan-ingestion.md`      | ingesting plan markdown into the task graph                          |
| `docs/agent-context/AGENTS-cli-reference.md`       | looking up or changing a godmode subcommand                          |
| `docs/agent-context/AGENTS-pipelines.md`           | starting, advancing, or running a pipeline                           |
| `docs/agent-context/AGENTS-plugin-layout.md`       | changing the plugin manifest, skills directory, or agents directory  |
| `docs/agent-context/AGENTS-opencode-parity.md`     | changing an agent, command, or hook that has an OpenCode projection  |
| `docs/agent-context/AGENTS-opencode-gates.md`      | committing a change to .opencode/ or the hook registry               |
| `docs/agent-context/AGENTS-gotchas.md`             | debugging a surprising failure in plan, dispatch, task, or hook code |
| `docs/agent-context/AGENTS-ci.md`                  | checking CI status for the current branch                            |
| `docs/agent-context/AGENTS-subagent-guardrails.md` | dispatching subagents or running parallel worktree work              |
| `docs/agent-context/AGENTS-nushell.md`             | writing or editing hook scripts                                      |
