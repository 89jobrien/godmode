# Why qual:allow(dry) Produces ORPHAN_SUPPRESSION for DEAD_CODE

## Root Cause

`qual:allow(dry)` does not suppress `DEAD_CODE` findings. This is a documented limitation:
`DEAD_CODE` is reported under the DRY dimension but is not suppressible via inline comments.
When rustqual scans `src/lib.rs`, it finds the `qual:allow(dry)` annotations on lines 19 and 24
but cannot match them to any finding they actually suppress — because they cannot suppress
`DEAD_CODE`. The unmatched annotations are then reported as `ORPHAN_SUPPRESSION`.

In short: `qual:allow(dry)` only suppresses duplicate-code and boilerplate findings, not
dead-code findings.

## Why the Functions Are Flagged as DEAD_CODE (testonly)

rustqual performs single-crate analysis. It cannot trace cross-crate call graphs. `node_count`
and `edge_count` are `pub` and used by another crate in the workspace, but rustqual cannot see
that usage. From its perspective, the only callers it can find are within the same crate — and
those callers are test code — so it labels both functions `(testonly)`.

## The Fix

Remove the `qual:allow(dry)` comments (they are causing ORPHAN_SUPPRESSION without helping),
and instead disable dead-code detection in `rustqual.toml`:

```toml
[duplicates]
detect_dead_code = false  # pub API consumed cross-crate; single-crate analysis cannot see it
```

This disables `DEAD_CODE` detection for the entire crate. That is appropriate here because all
the `DEAD_CODE (testonly)` findings are false positives caused by cross-crate usage being
invisible to rustqual.

After making this change, re-run rustqual. The four findings (two `DEAD_CODE`, two
`ORPHAN_SUPPRESSION`) should all be gone.

## Summary

| Finding                                              | Cause                                                                | Fix                                           |
| ---------------------------------------------------- | -------------------------------------------------------------------- | --------------------------------------------- |
| `DEAD_CODE (testonly)` on `node_count`, `edge_count` | Cross-crate callers invisible to rustqual                            | `detect_dead_code = false` in `rustqual.toml` |
| `ORPHAN_SUPPRESSION` on lines 19, 24                 | `qual:allow(dry)` cannot match `DEAD_CODE`; annotation has no effect | Remove the `qual:allow(dry)` comments         |
