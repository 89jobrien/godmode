---
name: ideate
allowed-tools:
- Bash
- Read
- Glob
- Grep
max-turns: 20
---
## Rules

- Always run `git branch --show-current` before any commit. If on main, STOP.
- Never use `--no-verify` on git commits.
- Conventional commits: `feat(<crate>):`, `fix(<crate>):`, `refactor(<crate>):`.
- Run quality gates through `taskit`, never raw `cargo`.
- Do NOT run gates by hand before committing or pushing. The global git hooks run format
  check, clippy on affected crates, tests, and the secret scan. Fix what a hook reports
  instead of re-running it. One exception: a format failure is detected but not fixed, so
  run `taskit check fmt` and `git add` the result.
- 3-attempt rule: after 3 failed attempts, write `BLOCKED.md` with the attempts
  and root cause, then stop. Do not continue patching.
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
