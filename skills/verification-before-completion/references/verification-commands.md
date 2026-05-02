# Verification Commands Reference

## Per-Crate Gate

```bash
cargo nextest run -p <crate>
cargo clippy -p <crate> -- -D warnings
cargo fmt -p <crate> --check
git log --oneline -3
```

All four must pass. Do not claim done until they do.

## Workspace Gate

```bash
cargo nextest run --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --all --check
```

## Subagent Verification

Never trust a subagent's self-report. After any subagent claims completion:

```bash
git log --oneline -5          # commits must exist
cargo nextest run --workspace # must pass independently
```

Empty commit log = subagent did not finish.

## Common False "Done" States

| Claim            | What to verify                               |
| ---------------- | -------------------------------------------- |
| "Tests pass"     | Run them now — not earlier, not cached       |
| "Clippy clean"   | Run `cargo clippy -- -D warnings`            |
| "It's committed" | Run `git log --oneline -3`                   |
| "Fix is in"      | Read the diff — is the change actually there |
| "Should work"    | Run it and confirm the output                |

## Exit Codes

A command that exits 0 is not sufficient evidence. Check:

- Did it actually run the test suite? (0 tests = 0 failures ≠ passing)
- Did `cargo check` succeed but `cargo test` wasn't run?
- Is the crate filter correct (`-p <crate>`)?
