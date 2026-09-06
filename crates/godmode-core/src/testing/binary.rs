//! Helpers for locating compiled binaries from integration tests.

use anyhow::{Context, Result};
use std::path::PathBuf;

/// Resolves a Cargo-built integration-test binary from its package target name.
pub fn binary_path(name: &str) -> Result<PathBuf> {
    let key = format!("CARGO_BIN_EXE_{}", name.replace('-', "_"));
    std::env::var_os(&key)
        .map(PathBuf::from)
        .with_context(|| format!("Cargo binary environment variable {key} is not set"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binary_path_resolves_cargo_bin_env() {
        let _guard = crate::testing::env::TestContext::builder()
            .env(
                "CARGO_BIN_EXE_godmode_test_helper",
                "/tmp/godmode-test-helper",
            )
            .build();

        assert_eq!(
            binary_path("godmode-test-helper").unwrap(),
            std::path::PathBuf::from("/tmp/godmode-test-helper")
        );
        assert!(binary_path("missing-test-helper").is_err());
    }
}
