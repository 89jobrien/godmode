# Pattern 3: Score mismatch (workspace vs per-crate)

Read this before comparing rustqual scores from a workspace scan against scores from per-crate scans, or before saving a baseline.

## Symptom

```bash
rustqual .              # reports 74.2%
rustqual crates/core    # reports 91.0%
rustqual crates/cli     # reports 68.0%
# 91 + 68 ≠ 74, and neither matches the workspace total
```

## Why this happens

Workspace scan (`rustqual .`) includes:

- All `.rs` files at the workspace root
- Re-export shims
- Files not inside any crate's `src/`

Per-crate scans only include that crate's `src/`. Per-crate totals do not sum to the
workspace total.

## Rule

Pick one scope and stick with it. Use `--save-baseline` to lock the method:

```bash
rustqual . --save-baseline .ctx/rustqual-baseline.json      # workspace scope
# OR
rustqual crates/core --save-baseline .ctx/core-baseline.json  # per-crate scope
```

Never compare a workspace score against a per-crate score — they measure different
things.
