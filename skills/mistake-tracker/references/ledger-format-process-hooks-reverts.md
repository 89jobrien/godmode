# Ledger format: process, hook, and revert entries

Read this when writing a process error, hook false positive, or revert entry to the mistake ledger, to copy the entry shape.

```yaml
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
