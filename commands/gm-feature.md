---
name: feature
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
- Cargo gates before committing: `cargo fmt --all`, `cargo clippy --workspace -- -D warnings`,
  `cargo nextest run --workspace`.
- 3-attempt rule: if a test or fix fails 3 times, stop and report the root cause.
  Do not continue patching.
- Run `cargo fmt --all` then re-stage before committing — the PostToolUse hook
  runs fmt automatically but does not stage.
- Commits are signed via SSH key through 1Password. If signing fails, tell the
  user to unlock 1Password — do not change git config.
- Scratch files go in `.ctx/_WORKING_DIR/`.


Full feature lifecycle from idea to committed code.
Do not write code until the user has approved the proposed approach. Pause before
task-driven development and report the plan path first.

## Workflow — pipeline: feature

1. Run godmode:brainstorm [optional] — Explore context, ask clarifying questions one at a time, and propose 2-3 named approaches. Wait for explicit user approval before continuing.
2. Run godmode:context-map — Map relevant files, dependencies, tests, reference patterns, and risks.
3. Run godmode:writing-plans — Scaffold the plan file and break the work into 2-5 minute tasks with a failing test, implementation, GREEN verification, and commit cycle.
4. Run godmode:task-management — Record the approved plan as an ordered task graph.
5. Run godmode:task-driven-development [repeat per task] — Execute the plan task by task using strict RED/GREEN/refactor cycles.
6. Run godmode:code-review — Review all severity levels in one pass and fix accepted findings.
7. Run godmode:verification-before-completion — Require every project gate to pass before shipping.
8. Run godmode:cap — Commit and push only after verification succeeds.
9. Run godmode:pr-author [optional] — Create the pull request when requested.
10. Run godmode:merge [optional] — Merge only after required checks and approvals pass.
