---
description: "Run the full feature lifecycle for this idea"
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

Run the full feature lifecycle for this idea: $ARGUMENTS
Treat `$ARGUMENTS` as descriptive text. If empty, ask for the feature idea and stop.
1. Run godmode:brainstorm — explore context, ask clarifying questions one at a time,
   propose 2-3 named approaches. Wait for explicit user approval before continuing.
2. Run godmode:design — produce typed API sketch, data flow, and component ownership.
   Requires brainstorm output. Present in sections, get feedback on each.
3. Run godmode:writing-plans — scaffold the plan file, break into 2-5 minute tasks,
   each with failing test → implement → verify GREEN → commit cycle.
4. Run godmode:ingest with the exact generated plan path — load and validate the task graph.
5. Run godmode:task-driven-development — execute the plan task by task.
6. Run godmode:verification-before-completion — all gates green before shipping.
7. Run godmode:cap — commit and push.
Do NOT write any code until the user has approved the design from step 1.
Pause for user confirmation before step 4. Report the plan file path after step 3.
