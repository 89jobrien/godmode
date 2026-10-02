---
name: debug-loop
allowed-tools:
- Bash
- Read
- Edit
- Write
- Glob
- Grep
max-turns: 40
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


Systematically debug a failure, verify the fix, and commit.
1. Run godmode:systematic-debugging — build a fast, deterministic feedback loop that
   triggers the bug before anything else. Then read the full error, check environment and
   recent changes, rank 3–5 falsifiable hypotheses, instrument one variable at a time, and
   apply the minimal fix at the correct test seam.
   Never hypothesise without a loop that triggers the bug.
   3-attempt rule: if 3 sequential fixes all fail, write BLOCKED.md and stop.
2. Run godmode:doublecheck — independent verification of the fix; confirm the root
   cause was addressed, not just the symptom.
3. Run godmode:verification-before-completion — all gates green before committing.
4. Run godmode:cap — commit with message describing the root cause and fix.
Do not skip doublecheck even if systematic-debugging feels thorough — it catches
fixes that address the symptom but leave the root cause intact.
