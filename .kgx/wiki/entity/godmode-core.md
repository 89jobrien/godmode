# Godmode Core

`godmode-core` owns task-graph domain logic, session orchestration, and external-tool integrations; `godmode-cli` delegates behavior to its public modules (`crates/godmode-core/src/lib.rs:1-49`). See [[Godmode Architecture Overview]].

Major components:

- model and graph: task schema, transitions, dependency resolution, persistence
- [[Session]]: lifecycle boundary for transitions, timing, trace emission, and save
- context and dispatch: machine-readable repository state and critical paths
- pipeline, workflow, and wave: orchestrated execution with explicit state contracts
- [[Integrations]]: external subprocess boundaries
- verify: composable Cargo-backed gates

The public module surface is explicit at `crates/godmode-core/src/lib.rs:9-49`, making schema and API compatibility relevant during refactors.
