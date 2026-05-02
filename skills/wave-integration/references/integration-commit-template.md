# Integration Commit Templates

## Standard wave integration commit

```
chore(integration): wave N integration — <N> branches merged

Branches:
- feat/branch-a (abc1234)
- feat/branch-b (def5678)
- feat/branch-c (ghi9012)

Tests: cargo test --workspace — 488 passed, 0 failed

Conflicts resolved: 2 files
- crates/foo/src/lib.rs: kept rename from main, added new variant from feat/branch-a
- Cargo.toml: took higher reqwest version (0.12.28) from feat/branch-b

Skipped: feat/branch-d (test failures after rebase — escalated)
```

## Zero-conflict integration commit

```
chore(integration): wave N integration — <N> branches merged, no conflicts

Branches:
- feat/branch-a (abc1234)
- feat/branch-b (def5678)

Tests: cargo test --workspace — 488 passed, 0 failed
```

## Partial integration (some branches failed)

```
chore(integration): wave N partial integration — <M>/<N> branches merged

Integrated:
- feat/branch-a (abc1234)
- feat/branch-b (def5678)

Not integrated (require manual resolution):
- feat/branch-c: rebase conflict in crates/bar/src/handler.rs — overlapping function bodies,
  cannot be composed automatically
- feat/branch-d: tests failed after rebase (test_exec_timeout — asserts old MockRuntime API)

Tests: cargo test --workspace — 488 passed, 0 failed (on integrated set)

Conflicts resolved: 1 file
- Cargo.toml: took higher tokio version (1.45.0) from feat/branch-b
```

## Rules for commit message

- Scope is always `integration`
- List every branch with its final SHA — never omit
- State test result explicitly (pass count or failure)
- List every conflict resolved with one-line reasoning
- List every failed/skipped branch and why — never silently omit
- Use past tense throughout
