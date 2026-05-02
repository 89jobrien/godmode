# Rust Debugging Checklist

## Environment First

Before looking at code, check these (most bugs are here):

```bash
echo $RUST_LOG
echo $RUST_BACKTRACE
op account list          # 1Password auth
env | grep -i api_key    # missing secrets
```

## Compiler Output

Read the full error, not just the first line. The Rust compiler often puts the real
cause several lines below the first `error[E...]` marker.

```bash
cargo check 2>&1 | less         # full output, no truncation
RUST_BACKTRACE=1 cargo nextest run -p <crate> -- <test>
RUST_BACKTRACE=full cargo nextest run -p <crate> -- <test>
```

## Async Failures

- Check tokio runtime context: `#[tokio::test]` required for async tests
- `spawn_blocking` vs `spawn` — CPU-bound work must use `spawn_blocking`
- `JoinHandle` dropped without `.await` silently cancels the task

## Lifetime Errors

Read the full borrow checker message. The fix is almost always one of:

1. Clone the value
2. Restructure to shorten the borrow
3. Use an owned type instead of a reference
4. Add a lifetime annotation

## Cross-Crate Failures

- Check feature flags in `Cargo.toml` — a feature may be enabled in one crate but not another
- `cfg(test)` gates on dependencies — `dev-dependencies` not available in integration tests
- Re-exports: is the type actually public from the crate root?

## Pattern: Working vs. Broken Comparison

```bash
git stash                        # stash current changes
cargo nextest run -p <crate>     # confirm working baseline
git stash pop                    # restore changes
cargo nextest run -p <crate>     # see what broke
git diff                         # review what changed
```

## 3-Failure Log

When stuck, write down:

1. Hypothesis tried
2. Change made
3. Result (error message verbatim)

After 3 entries with different results → surface to user. The architecture likely needs
redesign, not another patch.
