# Detection categories: test failures and process errors

Read this before classifying a test failure or process error as a recurring mistake.

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
