# Ledger format

Read this before creating or appending to `.ctx/godmode/memory-bank/mistakes.md`, or when checking that an existing entry has the required fields.

The file `.ctx/godmode/memory-bank/mistakes.md` uses Markdown with frontmatter:

```yaml
---
version: 1
last_updated: 2026-06-02
next_review: 2026-06-09
---

# Recurring Mistakes Ledger

## Clippy Lints

### clippy::too_many_arguments
- **Occurrences**: 5
- **Dates**: 2026-05-10, 2026-05-12, 2026-05-15
- **Affected**: godmode-core/src/session.rs (Session::start_task)
- **Prevention**: Refactor functions to use builder pattern or struct wrapper
- **Notes**: Pattern appears when Session methods grow in scope

## Test Failures

### runnable_returns_tasks (flake)
- **Occurrences**: 3
- **Dates**: 2026-05-08, 2026-05-11, 2026-05-14
- **Affected**: godmode-core tests (task graph resolution)
- **Prevention**: Fix race condition in test setup or add retry logic
- **Notes**: Fails intermittently; may indicate test isolation issue

## Process Errors

### Subagent commits to main
- **Occurrences**: 2
- **Dates**: 2026-05-06 (abc123d), 2026-05-11 (def456g)
- **Prevention**: Pre-commit hook to guard against commits to main; require confirmation
- **Notes**: Subagents sometimes forget to switch branches before committing

## Hook False Positives

### obfsck: localhost IPs in test fixtures
- **Occurrences**: 4
- **Dates**: Allowlist entries 2026-05-01, 2026-05-06, 2026-05-12
- **Affected**: tests/fixtures/ (various)
- **Prevention**: Add per-file `allowlist:` directives in test files
- **Notes**: Overly broad IP pattern catches test localhost addresses

## Reverts

### revert: godmode task start breaks on empty graph
- **Occurrences**: 1
- **Dates**: 2026-05-13 (rev123)
- **Prevention**: Add integration test for empty graph edge case
- **Notes**: Review missed that empty graph returns error instead of empty list
```
