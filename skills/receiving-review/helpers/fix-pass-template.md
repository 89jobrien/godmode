# Review Fix Pass

Use this to track a single review cycle. Fill in before touching any code.

## Source

Reviewer: `<sentinel | human | clippy | obfsck>`
Date: YYYY-MM-DD
PR/branch: `<branch-name>`

## Triage

| #   | Finding | File:line | Class | Action |
| --- | ------- | --------- | ----- | ------ |
| 1   |         |           |       |        |
| 2   |         |           |       |        |

Classes: `valid` | `false-positive` | `scope-creep` | `disagreement`

## Fix Plan

Order: blocking → suggestions → nitpicks. List each fix one line:

1. [ ] `<file:line>` — <what to change>
2. [ ] `<file:line>` — <what to change>

## Allowlist Entries Needed

- [ ] `<pattern>` in `<file>` — reason: <why it's a false positive>

## Verification After Fixes

```bash
cargo nextest run --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --all --check
```

- [ ] All pass
- [ ] `godmode:verification-before-completion` run
- [ ] Single fix commit created
