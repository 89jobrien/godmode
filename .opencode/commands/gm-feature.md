---
description: Full feature lifecycle from idea to committed code.
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


Full feature lifecycle from idea to committed code.
1. Run godmode:brainstorm — explore context, ask clarifying questions one at a time,
   propose 2-3 named approaches. Wait for explicit user approval before continuing.
2. Run godmode:design — produce typed API sketch, data flow, and component ownership.
   Requires brainstorm output. Present in sections, get feedback on each.
3. Run godmode:writing-plans — scaffold the plan file, break into 2-5 minute tasks,
   each with failing test → implement → verify GREEN → commit cycle.
4. Run godmode:task-driven-development — execute the plan task by task.
5. Run godmode:verification-before-completion — all gates green before shipping.
6. Run godmode:cap — commit and push.
Do NOT write any code until the user has approved the design from step 1.
Pause for user confirmation before step 4. Report the plan file path after step 3.
