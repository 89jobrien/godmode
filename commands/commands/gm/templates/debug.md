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
