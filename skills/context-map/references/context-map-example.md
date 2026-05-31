# Context Map Example — Adding `blocked_count` to SessionContext

## Files to Modify

| File                                 | Purpose               | Changes Needed                  |
| ------------------------------------ | --------------------- | ------------------------------- |
| `crates/godmode-core/src/context.rs` | SessionContext struct | add `blocked_count` field       |
| `crates/godmode-cli/src/main.rs`     | CLI handler           | emit new field in --json output |

## Dependencies (may need updates)

| File                             | Relationship                                  |
| -------------------------------- | --------------------------------------------- |
| `crates/godmode-core/src/lib.rs` | re-exports `SessionContext`                   |
| `hooks/scripts/session-start.nu` | calls `godmode context --json`, parses output |

## Test Coverage

| Test                                          | Covers                  |
| --------------------------------------------- | ----------------------- |
| `crates/godmode-core/src/context.rs` (inline) | `SessionContext::build` |
| `tests/conformance/src/context.rs`            | `--json` output shape   |

## Reference Patterns

| File                               | Pattern to Follow                          |
| ---------------------------------- | ------------------------------------------ |
| `crates/godmode-core/src/cache.rs` | struct serialised to JSON file, same shape |

## Risk

- [x] `SessionContext` is `pub` — all consumers listed above must be updated
- [x] `--json` output changes: update conformance test fixture
- [ ] No migration needed — field is additive with `#[serde(default)]`
