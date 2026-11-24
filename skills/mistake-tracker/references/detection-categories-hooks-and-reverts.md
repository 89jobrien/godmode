# Detection categories: hook false positives and reverts

Read this before classifying a hook false positive or a revert as a recurring mistake.

## 4. Hook False Positives

Same hook repeatedly blocking legitimate commits:

- Pre-commit obfsck flagging test fixture URLs as secrets
- Pre-commit pathspec pattern too broad, matching unintended files
- Hook regex matching variable names like `password` in test code

Example:

```text
Pattern: obfsck false positive on localhost IPs in test file
Occurrences: 4 (added to allowlist 2026-05-01, 2026-05-06, 2026-05-12)
Prevention: Consolidate allowlist entries; add per-line allow comments to tests
```

## 5. Reverts

Commits that were reverted because they broke something. Indicates a mistake made it
past initial review and testing. Root cause may be:

- Test coverage gap
- Review process gap
- Integration testing gap
- Race condition or environment-specific issue

Example:

```text
Pattern: revert: godmode task start breaks on empty graph
Occurrences: 1 (commit rev123, 2026-05-13)
Prevention: Add integration test for empty graph; add guard in start() function
```
