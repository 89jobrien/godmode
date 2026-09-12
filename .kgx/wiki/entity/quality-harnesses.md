# Quality Harnesses

`godmode-conformance` is a non-published workspace member that depends on [[Godmode Core]], Proptest, and Criterion (`tests/conformance/Cargo.toml:1-31`). Its `all_tests` entry point registers graph, plan, dispatch, wave, Crux, fixture, and plugin-structure suites (`tests/conformance/src/lib.rs:23-33`).

`xtask` centralizes pre-commit and CI gates. It invokes formatting, Clippy with denied warnings, nextest, conformance, plugin checks, generated index checks, release validation, and cargo-deny (`xtask/src/main.rs:21-123`).

The core verify module provides composable Cargo-backed nextest, Clippy, formatting, commit, and globstar steps (`crates/godmode-core/src/verify.rs:33-115`).
