# Inline suppressions

Read this before adding a qual:allow comment, or when a suppression produces an ORPHAN_SUPPRESSION finding.

## Inline Suppressions

Suppress specific findings with comments above the function:

```rust
// qual:allow(iosp) reason: "I/O boundary function"
fn read_data(path: &Path) -> Result<Data> { ... }
```

Available dimensions: `iosp`, `complexity`, `dry`, `srp`, `coupling`,
`test`, `architecture`.

A bare `// qual:allow` (no dimension) suppresses all dimensions.

**Gotchas**:

- `qual:allow(dry)` does NOT suppress DEAD_CODE findings — only duplicate
  and boilerplate findings. DEAD_CODE is reported under DRY but is not
  suppressible inline. Use `detect_dead_code = false` in config instead.
- Stale or mismatched suppressions produce ORPHAN_SUPPRESSION findings.
- The `max_suppression_ratio` config (default 0.05) warns if too many
  functions are suppressed.
