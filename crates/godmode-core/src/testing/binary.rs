//! Helpers for locating compiled binaries from integration tests.

use std::path::PathBuf;

use anyhow::{Context, Result};

/// Resolves a Cargo-built integration-test binary from its package target name.
///
/// # Errors
///
/// Returns an error when Cargo did not provide the target binary environment variable.
pub fn binary_path(name: &str) -> Result<PathBuf> {
    let key = format!("CARGO_BIN_EXE_{}", name.replace("-", "_"));
    std::env::var_os(&key)
        .map(PathBuf::from)
        .with_context(|| format!("Cargo binary environment variable {key} is not set"))
}

#[cfg(test)]
mod tests {
    #[test]
    fn binary_path_resolves_hyphenated_cargo_target_name() {
        let _guard = crate::testing::env::TestContext::builder()
            .env(
                "CARGO_BIN_EXE_godmode_test_helper",
                "/tmp/godmode-test-helper",
            )
            .build();
        assert_eq!(
            super::binary_path("godmode-test-helper").unwrap(),
            std::path::PathBuf::from("/tmp/godmode-test-helper")
        );
    }

    #[test]
    fn binary_path_reports_missing_cargo_target() {
        assert!(super::binary_path("missing-test-helper").is_err());
    }
}
