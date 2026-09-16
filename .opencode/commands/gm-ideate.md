---
description: Explore the repo for new feature ideas grounded in what the code already does and what it's missing.
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


Explore the repo for new feature ideas grounded in what the code already does and what it's missing.
1. Scan the repo structure: README, CLAUDE.md, Cargo.toml workspace members, top-level dirs.
2. Read entry points (lib.rs, main.rs, mod.rs) for each major component.
3. Grep for TODO, FIXME, HACK, unimplemented!(), todo!() across the workspace.
4. Check .github/workflows for CI gaps (missing lint steps, no benchmark job, etc.).
5. Look at existing skills/agents/commands for patterns that are half-implemented or
   that suggest natural extensions.
6. Synthesise findings into a prioritised idea list grouped by effort:
   - Quick Wins: small, self-contained, high signal-to-noise
   - Medium: meaningful new capability, 1-3 day scope
   - Large: architectural or cross-cutting, worth planning carefully
7. Present the list. Ask which idea to pursue. Hand off to godmode:brainstorm with the
   chosen idea and the evidence that motivated it.
Read code, do not invent. Every idea must trace to something observed in the repo.
Output is a prioritised idea list only — no implementation, no scaffolding, no code.
