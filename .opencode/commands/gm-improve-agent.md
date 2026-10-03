---
description: Identify recurring agent behaviour patterns and codify improvements into skills or agents.
subtask: false
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


Identify recurring agent behaviour patterns and codify improvements into skills or agents.
1. Run godmode:self-reflect — review recent sessions for friction, repeated corrections,
   and process gaps.
2. Run godmode:pattern-learner — extract recurring patterns from session traces and
   mistake ledger. Classify: best practice / anti-pattern / process gap.
3. Run godmode:agent-improvement-loop — for each high-signal pattern, propose a concrete
   change: new skill rule, updated agent description, or new command.
   Present proposed changes one at a time. Wait for approval before applying each.
4. Run godmode:agents-skill-save — write approved changes to the relevant skill or agent
   file. Validate frontmatter after each write.
5. Commit: feat(godmode): improve <skill/agent name> based on session patterns.
Every improvement must be grounded in observed behaviour — not hypothetical issues.
Do not modify a skill or agent file without showing the proposed diff and getting approval.
