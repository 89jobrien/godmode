//! Reusable test infrastructure primitives for hexagonal Rust workspaces.
//!
//! Gated behind `features = ["testing"]` so the CLI binary doesn't pull
//! these in.
//!
//! # Modules
//!
//! - [`conformance`] — LSP-enforcement suite: write cases once, run against
//!   N adapters of a port trait.
//! - [`audit`] — compile-time trait-bound checks, dep-allowlist auditing,
//!   golden-file snapshot comparisons.
//! - [`env`] — RAII environment variable isolation for tests.
//! - [`prop`] — proptest presets and serde round-trip assertion.
//! - [`seed`] — deterministic FNV-1a seed from test names.
//! - [`binary`] — locate compiled binaries from integration tests.

pub mod audit;
pub mod binary;
pub mod conformance;
pub mod env;
pub mod prop;
pub mod seed;
