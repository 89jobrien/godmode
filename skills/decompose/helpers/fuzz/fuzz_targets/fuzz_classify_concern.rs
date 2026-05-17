#![no_main]
use libfuzzer_sys::fuzz_target;

/// Mirrors classify_concern from analyze-diff.nu.
/// Must never panic on any input — returns a non-empty static str.
fn classify_concern(file: &str) -> &'static str {
    if file.contains("Cargo.toml") || file.contains("Cargo.lock") {
        "deps"
    } else if file.starts_with(".github/") {
        "ci"
    } else if file.contains("/tests/") || file.starts_with("tests/") {
        "tests"
    } else if file.contains("/benches/") || file.starts_with("benches/") {
        "benches"
    } else if file.ends_with(".md") || file.ends_with(".txt") || file.starts_with("docs/") {
        "docs"
    } else if file.contains("/examples/") || file.starts_with("examples/") {
        "examples"
    } else if file.ends_with(".nu") || file.ends_with(".sh") {
        "scripts"
    } else {
        "logic"
    }
}

fuzz_target!(|data: &[u8]| {
    // Accept any byte sequence — classify_concern must not panic on any valid UTF-8,
    // and must degrade gracefully on invalid UTF-8 (simply skip non-UTF-8 inputs).
    if let Ok(s) = std::str::from_utf8(data) {
        let concern = classify_concern(s);
        // Invariants that must always hold:
        assert!(!concern.is_empty(), "concern must be non-empty for input: {:?}", s);
        assert!(
            ["deps", "ci", "tests", "benches", "docs", "examples", "scripts", "logic"]
                .contains(&concern),
            "concern must be a known variant, got: {:?} for input: {:?}",
            concern,
            s
        );
    }
    // Non-UTF-8 input: no-op. classify_concern only operates on valid paths.
});
