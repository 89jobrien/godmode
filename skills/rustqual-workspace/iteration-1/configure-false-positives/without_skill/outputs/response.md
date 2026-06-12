Two separate issues are combining here.

## Wrong suppression tag for DEAD_CODE

`qual:allow(dry)` is the tag for the DRY (duplication) rule family, not DEAD_CODE. That mismatch
is why rustqual emits ORPHAN_SUPPRESSION — the directive exists but no `dry`-category finding is
present on those lines, so the suppression is considered orphaned.

To suppress DEAD_CODE findings, the correct tag is `qual:allow(dead_code)` (matching the finding
name, lowercased and underscored). Change lines 19 and 24 to:

```rust
// qual:allow(dead_code)
pub fn node_count(...) { ... }
```

## Why DEAD_CODE fires at all for pub functions used cross-crate

The `(testonly)` annotation in the findings is the real underlying issue. rustqual marks a function
`testonly` when it detects the only call sites are inside `#[cfg(test)]` blocks or test crates —
even if the function itself is `pub`. If the consuming crate references these functions only from
its own test modules, rustqual still classifies them as dead from a production-use standpoint.

To resolve this cleanly, two options:

1. **Suppress with the correct tag** — use `qual:allow(dead_code)` if the test-only usage is
   intentional and you want to silence the finding permanently.

2. **Gate functions under `#[cfg(test)]`** — if these functions genuinely only exist to support
   tests in the other crate, annotating them explicitly (e.g., via a `testonly` feature flag or
   cfg gate) removes the ambiguity and the finding disappears without needing a suppression.

## Summary

| Problem              | Cause                                    | Fix                                              |
| -------------------- | ---------------------------------------- | ------------------------------------------------ |
| ORPHAN_SUPPRESSION   | Wrong rule tag (`dry` vs `dead_code`)    | Change to `qual:allow(dead_code)`                |
| DEAD_CODE (testonly) | Functions only called from test contexts | Accept suppression, or gate under `#[cfg(test)]` |
