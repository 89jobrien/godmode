# Detection categories

Read this before classifying a recurring failure, or when deciding which of the five categories a new pattern belongs to.

## 1. Clippy Lint Recurrence

Repeated clippy warnings with the same lint ID appearing across multiple sessions or
commits. Indicates a pattern the codebase is prone to that needs either:

- Systemic suppression (if false positive or accepted)
- Refactoring (if genuine pattern to eliminate)
- Compiler upgrade (if lint behavior changed)

Example:

```text
Pattern: clippy::too_many_arguments
Occurrences: 5 (sessions 2026-05-10, 2026-05-12, 2026-05-15)
Affected: crates/godmode-core/src/session.rs
Prevention: Apply deny rule or refactor functions to use builder pattern
```

## 2. Test Failures

Same test failing across different branches or session runs. Indicates:

- Flaky test (race condition, timing dependency)
- Environment-specific failure (only fails on CI, only on macOS, etc.)
- Real bug that resurfaces on different code paths
- Test that needs isolation or setup fixes

Example:

```text
Pattern: runnable_returns_tasks
Occurrences: 3 (2026-05-08, 2026-05-11, 2026-05-14)
Affected: godmode-core integration tests
Prevention: Add test-level retry or fix race condition in task graph locking
```

## 3. Process Errors

Repeated mistakes in git/branch/merge workflow:

- Committing to main instead of feature branch (subagent error)
- Force-pushing when merge was intended
- Stash drops or data loss
- Merging out-of-order, causing integration conflicts

Example:

```text
Pattern: subagent commit to main
Occurrences: 2 (commits abc123d, def456g)
Prevention: Add git branch guard hook; require explicit confirmation for main commits
```

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
