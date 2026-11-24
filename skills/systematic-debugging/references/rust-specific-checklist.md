# Rust-specific debugging checklist

Read this before debugging a panic, borrow or lifetime error, async failure, cross-crate failure, or flaky test in a Rust workspace.

## Rust-Specific Checklist

- `RUST_BACKTRACE=1` or `RUST_BACKTRACE=full` for panics
- `RUST_LOG=debug` for tracing output
- `taskit check compile` before `taskit test run` — catch compile errors cheaply
- `cargo clippy -p <crate> -- -D warnings` — warnings often point at the root cause
- Lifetime and borrow errors: read the full compiler message, not just the first line
- Async failures: check executor context (tokio runtime not entered, etc.)
- Cross-crate failures: check feature flags and conditional compilation gates
- Flaky tests: `cargo nextest run --retries 0` surfaces hidden order dependencies; then
  `--test-threads 1` to confirm it is parallelism, not logic
