# Build and test commands

Read this before running any build, test, lint, format, coverage, or audit gate.

```bash
taskit check quick                          # fast loop: fmt-check + lint + affected-crate tests
taskit check ci                             # full local CI, summary table
taskit check lint                           # clippy, -D warnings, all targets
taskit check lint --crate-name godmode-core # one crate
taskit check fmt --check                    # format check
taskit check fmt                            # format
taskit check compile                        # compile test binaries without running
taskit test run                             # nextest, workspace
taskit test run --crate-name godmode-core   # one crate
taskit test coverage                        # coverage with threshold
taskit test proptest --crate-name godmode-conformance
taskit test bench --crate-name godmode-conformance
taskit protocol audit                       # cargo-deny: advisories, licenses, bans
taskit protocol freshness                   # outdated dependency check
taskit check pre-commit                     # what the pre-commit hook runs

just conformance                            # plugin structure + subcommand + consistency checks
```

`taskit` auto-discovers the workspace from `Cargo.toml`, so no `taskit.toml` is required.
This table is a reference for ad-hoc runs — do not run these before committing or pushing,
because the global git hooks already gate format, clippy, and tests.

### Commands with no taskit equivalent

Keep `cargo` for these — taskit exposes no wrapper, and neither is a quality gate:

```bash
# Binary execution
cargo run -p godmode-conformance --bin run-conformance -- --verbose   # conformance suite

# Single-test filter (no passthrough on `taskit test run`)
cargo nextest run -E 'test(runnable_returns_tasks)'
cargo test -p godmode-core runnable_returns_tasks
```
