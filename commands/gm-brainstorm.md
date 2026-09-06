---
description: "Explore and converge on a design for this topic before implementation begins"
allowed-tools:
  - Bash
  - Read
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

Explore and converge on a design for this topic before implementation begins: $ARGUMENTS
Treat `$ARGUMENTS` as descriptive text. If empty, ask for the topic and stop.
Follow godmode:brainstorm exactly:

1. Read the relevant CLAUDE.md, Cargo.toml, and any analogous existing code first.
2. Ask one clarifying question at a time until scope and constraints are clear.
3. Propose 2-3 named approaches — each with a 2-3 sentence description and the key trade-off.
4. Present the design in sections: get feedback on each before moving to the next.
5. Once the user approves an approach, summarise: approved approach name, goal statement,
   explicit constraints, and out-of-scope items.
6. Stop and report that the next command is `/gm:design`. Do not invoke godmode:design here.
   Do NOT write any code, scaffold any files, or invoke any implementation skill until
   the user has explicitly approved a design.
