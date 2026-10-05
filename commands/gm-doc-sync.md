---
name: doc-sync
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


Detect drift between documentation and code. Read-only — report findings, do not fix.
Follow godmode:doc-sync exactly:
1. Check CLI surface: run --help for each subcommand, compare against documented flags.
2. Check crate/module surface: compare ls crates/, pub mod, pub fn/struct/enum/trait
   against CLAUDE.md and README tables.
3. Check skills and agents: compare ls skills/*/SKILL.md and ls agents/*.md against
   all skill/agent tables in docs.
4. Check file paths: extract every literal path from all docs, confirm each exists.
5. Check cross-doc consistency: same feature described in multiple docs must agree.
Report findings grouped by severity: Blocking → Suggestion → Nitpick.
Suggest minimal fixes for Blocking findings. Do not apply any fixes here.
Hand off to godmode:doc-writer for missing docs or godmode:doc-review after fixes.
