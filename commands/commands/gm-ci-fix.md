---
name: ci-fix
allowed_tools:
  - Bash
  - Read
  - Edit
  - Write
  - Glob
  - Grep
max_turns: 25
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

Diagnose and fix the latest CI failure on the current branch.
Follow godmode:ci-fix exactly:

1. Run skills/ci-fix/helpers/fetch-failure.nu to get logs.
2. Classify the root cause (compile_error, test_failure, clippy_warning, fmt_check,
   pre_commit_hook, runner_environment, false_positive, dependency_issue).
3. Apply the minimum targeted fix for that class only.
4. Verify locally: cargo check, nextest, clippy, fmt.
5. Commit with message "fix(ci): <root cause summary>" and push.
6. Report the run ID, class, fix applied, and new commit SHA.
   Do NOT switch self-hosted runners, change model names, or use --no-verify.
