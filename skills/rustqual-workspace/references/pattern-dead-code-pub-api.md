# Pattern 1: DEAD_CODE on pub API

Read this before suppressing or configuring a DEAD_CODE finding on a `pub` function that another crate in the workspace uses.

## Symptom

```text
DEAD_CODE  src/lib.rs:20  node_count  (testonly)
DEAD_CODE  src/lib.rs:25  edge_count  (testonly)
```

These functions are `pub`, used by another crate, but rustqual only sees the unit
tests in this crate calling them — hence `(testonly)`.

## Wrong fix

Adding `// qual:allow(dry)` above the functions does NOT suppress DEAD_CODE. It
suppresses duplicate/boilerplate DRY findings only. The result:

```text
ORPHAN_SUPPRESSION  src/lib.rs:19  qual:allow(dry) has no matching finding
```

## Correct fix

Disable dead code detection in `rustqual.toml`:

```toml
[duplicates]
detect_dead_code = false
```

Only do this when **all** DEAD_CODE findings are cross-crate false positives. If
some are genuine dead code, use `exclude_files` to carve out false-positive files
instead:

```toml
exclude_files = ["src/graph_api.rs"]  # pub API consumed by graph-cli crate
```
