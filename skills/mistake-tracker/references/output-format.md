# Output format for findings

Read this before reporting recurring mistakes to the user, so the summary uses the standard headings and next-steps line.

When reporting findings to the user, use:

```text
## Recurring Mistakes Detected

### Clippy Lints (2 patterns)
- clippy::too_many_arguments (5 occurrences)
- clippy::needless_borrow (2 occurrences)

### Test Failures (1 pattern)
- runnable_returns_tasks (3 occurrences, flaky)

### Process Errors (1 pattern)
- subagent commits to main (2 occurrences)

### Hook False Positives (1 pattern)
- obfsck: localhost IPs (4 occurrences, allowlisted)

### Reverts (1 pattern)
- task start breaks on empty graph (1 occurrence)

**Next Steps**: Review `.ctx/godmode/memory-bank/mistakes.md` for prevention strategies and
see which can be implemented before the next release.
```
