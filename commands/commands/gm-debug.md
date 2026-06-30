---
name: debug
allowed_tools:
  - Bash
  - Read
  - Edit
  - Write
  - Glob
  - Grep
max_turns: 30
---

## Rules

- Read the full error output before proposing any fix. Do not skim.
- Check environment variables and secrets resolution FIRST before investigating
  code-level causes.
- State one specific hypothesis before touching any code.
- Check recent changes: `git log --oneline -5`, `git diff HEAD~1`.
- 3-attempt rule: if 3 sequential fix attempts all fail, stop and report the
  architectural issue. Write BLOCKED.md and escalate.
- Never use `--no-verify` on git commits.
- Run `git branch --show-current` before any commit. If on main, STOP.

Systematically debug a failing test or unexpected behaviour.
Follow godmode:systematic-debugging exactly:

1. Run skills/systematic-debugging/helpers/debug-session.nu <crate> [test_name].
2. Parse the full error — do not skim.
3. Check recent changes (git log, git diff HEAD~1).
4. State one specific hypothesis before touching any code.
5. Write a failing test that captures the bug (if one doesn't exist).
6. Implement the single fix. Verify with cargo nextest + clippy.
   3-failure rule: if 3 sequential attempts all fail, stop and report the architectural
   issue — do not continue patching.
