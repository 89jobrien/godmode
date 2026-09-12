# Godmode Architecture Overview

Functional/session baseline `3589433cc3c907c3a60085c830549bee375e2838` is a Rust 2024 workspace with four members: `godmode-core`, `godmode-cli`, `tests/conformance`, and `xtask` (`Cargo.toml:1-9`).

The `godmode` binary is a Clap presentation layer that delegates to [[Godmode Core]]. Task commands open [[Session]], while explicit execution and synchronization cross [[Integrations]]. Runtime contracts are summarized in [[Persisted State]], and development gates in [[Quality Harnesses]].

Core public components include model and graph modules, session orchestration, context and dispatch, pipeline, workflow, wave, verification, and external-tool adapters (`crates/godmode-core/src/lib.rs:9-49`).

Evidence was refreshed from repository source at baseline commit `3589433cc3c907c3a60085c830549bee375e2838`.
