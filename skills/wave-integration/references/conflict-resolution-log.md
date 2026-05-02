# Conflict Resolution Log — Wave N Integration

**Date:** YYYY-MM-DD
**Base branch:** main
**Branches merged:** feat/a, feat/b, feat/c

---

## Conflict Resolution Table

| File                        | Branch   | Main-side intent                     | Branch-side intent                           | Resolution                                           |
| --------------------------- | -------- | ------------------------------------ | -------------------------------------------- | ---------------------------------------------------- |
| `crates/foo/src/lib.rs`     | `feat/a` | Added `FooError::Timeout` variant    | Renamed `FooError::Io` → `FooError::IoError` | Applied rename; added `Timeout` variant after        |
| `Cargo.toml`                | `feat/b` | `reqwest = "0.12.27"`                | `reqwest = "0.12.28"`                        | Took higher version `0.12.28`                        |
| `crates/bar/src/handler.rs` | `feat/c` | Added `handle_exec` fn (new feature) | Moved `handle_run` to separate file          | Kept file split; added `handle_exec` in new location |

---

## Branches Integrated

- `feat/a` → merged at `abc1234`
- `feat/b` → merged at `def5678`
- `feat/c` → merged at `ghi9012`

## Failed / Skipped

- `feat/d` — tests failed after rebase (see notes)

## Notes

- `feat/d`: `crates/baz/src/lib.rs` test `test_timeout_behavior` failed after rebase onto main.
  Root cause: `feat/d` assumed `MockRuntime::new()` returns `Ok(Self)` but main changed it to
  infallible. Fixed by unwrapping in test; re-ran `cargo test --workspace` — passes.
