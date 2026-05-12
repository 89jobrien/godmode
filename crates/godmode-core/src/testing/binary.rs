//! Helpers for locating compiled binaries from integration tests.

use std::path::PathBuf;

/// Find a compiled binary relative to the currently-running test binary.
///
/// Integration tests run from `target/<profile>/deps/<test-name>`. The built
/// binary lives at `target/<profile>/<name>`. This function navigates up two
/// levels then appends the binary name.
///
/// # Panics
///
/// Panics if `current_exe()` cannot be resolved or the binary doesn't exist.
pub fn find_test_binary(name: &str) -> PathBuf {
    let current = std::env::current_exe().expect("failed to resolve current_exe()");

    let profile_dir = current
        .parent() // deps/
        .and_then(|p| p.parent()) // <profile>/
        .unwrap_or_else(|| panic!("unexpected test binary path: {}", current.display()));

    let binary = profile_dir.join(name);
    assert!(
        binary.exists(),
        "binary '{name}' not found at {}.\n\
         Run `cargo build -p {name}` first.",
        binary.display(),
    );
    binary
}
