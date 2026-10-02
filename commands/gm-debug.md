---
name: debug
allowed-tools:
- Bash
- Read
- Edit
- Write
- Glob
- Grep
max-turns: 30
---
## Rules

- Build a fast, deterministic feedback loop that triggers the bug before anything else.
  Never hypothesise without one — if you cannot build one, stop and ask for environment
  access, a captured artifact, or permission to add temporary instrumentation.
- Read the full error output before proposing any fix. Do not skim.
- Check environment variables and secrets resolution FIRST before investigating
  code-level causes.
- Rank 3–5 falsifiable hypotheses before touching any code. Each must state the
  prediction it makes. Instrument one variable at a time.
- Check recent changes: `git log --oneline -5`, `git diff HEAD~1`.
- 3-attempt rule: if 3 sequential fix attempts all fail, stop and report the
  architectural issue. Write BLOCKED.md and escalate.
- Never use `--no-verify` on git commits.
- Run `git branch --show-current` before any commit. If on main, STOP.


Systematically debug a failing test or unexpected behaviour.
Follow godmode:systematic-debugging exactly:
1. Run skills/systematic-debugging/helpers/debug-session.nu <crate> [test_name].
   If that does not yield a fast, deterministic loop that triggers the bug, build one
   before continuing — see the Phase 0 ladder in the skill.
2. Parse the full error — do not skim.
3. Check recent changes (git log, git diff HEAD~1).
4. Rank 3–5 falsifiable hypotheses before touching any code, each stating the prediction
   it makes. Instrument one variable at a time.
5. Write a failing test at the correct seam that captures the bug (if one doesn't exist).
6. Implement the single fix. Verify with cargo nextest + clippy.
3-failure rule: if 3 sequential attempts all fail, stop and report the architectural
issue — do not continue patching.
