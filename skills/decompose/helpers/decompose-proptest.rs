#!/usr/bin/env rust-script
//! Property tests for the decompose grouping and coverage logic.
//!
//! Tests the deterministic parts: concern classification, crate mapping, coverage verification.
//! Run with: cargo test -p godmode-conformance decompose
//!
//! ```cargo
//! [dependencies]
//! proptest = "1"
//! ```

use proptest::prelude::*;

// ── Concern classification ────────────────────────────────────────────────────

/// Mirrors the classify_concern logic from analyze-diff.nu.
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

/// Coverage check: union of all split file lists equals source file list.
fn verify_coverage(source: &[String], splits: &[Vec<String>]) -> CoverageResult {
    let mut covered: Vec<&str> = Vec::new();
    for split in splits {
        for f in split {
            covered.push(f.as_str());
        }
    }

    let orphaned: Vec<&str> = source
        .iter()
        .filter(|f| !covered.contains(&f.as_str()))
        .map(|f| f.as_str())
        .collect();

    // Detect duplicates
    let mut seen = std::collections::HashSet::new();
    let duplicated: Vec<&str> = covered
        .iter()
        .filter(|f| !seen.insert(*f))
        .copied()
        .collect();

    CoverageResult {
        ok: orphaned.is_empty() && duplicated.is_empty(),
        orphaned: orphaned.iter().map(|s| s.to_string()).collect(),
        duplicated: duplicated.iter().map(|s| s.to_string()).collect(),
    }
}

#[derive(Debug, PartialEq)]
struct CoverageResult {
    ok: bool,
    orphaned: Vec<String>,
    duplicated: Vec<String>,
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_cargo_toml() {
        assert_eq!(classify_concern("crates/foo/Cargo.toml"), "deps");
        assert_eq!(classify_concern("Cargo.lock"), "deps");
    }

    #[test]
    fn classify_github_workflows() {
        assert_eq!(classify_concern(".github/workflows/ci.yml"), "ci");
    }

    #[test]
    fn classify_tests() {
        assert_eq!(classify_concern("crates/foo/tests/integration.rs"), "tests");
        assert_eq!(classify_concern("tests/smoke.rs"), "tests");
    }

    #[test]
    fn classify_docs() {
        assert_eq!(classify_concern("README.md"), "docs");
        assert_eq!(classify_concern("docs/architecture.md"), "docs");
    }

    #[test]
    fn classify_logic_default() {
        assert_eq!(classify_concern("crates/foo/src/lib.rs"), "logic");
        assert_eq!(classify_concern("src/main.rs"), "logic");
    }

    #[test]
    fn coverage_ok_when_splits_cover_source() {
        let source = vec!["a.rs".to_string(), "b.rs".to_string(), "c.rs".to_string()];
        let splits = vec![
            vec!["a.rs".to_string()],
            vec!["b.rs".to_string(), "c.rs".to_string()],
        ];
        let result = verify_coverage(&source, &splits);
        assert!(result.ok);
        assert!(result.orphaned.is_empty());
        assert!(result.duplicated.is_empty());
    }

    #[test]
    fn coverage_detects_orphaned_file() {
        let source = vec!["a.rs".to_string(), "b.rs".to_string(), "missing.rs".to_string()];
        let splits = vec![
            vec!["a.rs".to_string()],
            vec!["b.rs".to_string()],
        ];
        let result = verify_coverage(&source, &splits);
        assert!(!result.ok);
        assert_eq!(result.orphaned, vec!["missing.rs"]);
    }

    #[test]
    fn coverage_detects_duplicated_file() {
        let source = vec!["a.rs".to_string(), "b.rs".to_string()];
        let splits = vec![
            vec!["a.rs".to_string(), "b.rs".to_string()],
            vec!["b.rs".to_string()],  // b.rs appears twice
        ];
        let result = verify_coverage(&source, &splits);
        assert!(!result.ok);
        assert!(result.duplicated.contains(&"b.rs".to_string()));
    }
}

// ── Property tests ────────────────────────────────────────────────────────────

proptest! {
    /// Every file in the source appears in exactly one split when coverage is ok.
    #[test]
    fn prop_coverage_ok_iff_no_orphan_no_dup(
        source in prop::collection::vec("[a-z]{1,8}\\.rs", 1..20usize),
        seed in 0usize..100usize,
    ) {
        // Build a valid partition of source
        let n = (seed % 4) + 1; // 1–4 splits
        let mut splits: Vec<Vec<String>> = vec![vec![]; n];
        for (i, file) in source.iter().enumerate() {
            splits[i % n].push(file.clone());
        }
        let result = verify_coverage(&source, &splits);
        prop_assert!(result.ok, "valid partition should pass coverage: {:?}", result);
    }

    /// Dropping a file from splits always produces an orphan.
    #[test]
    fn prop_orphan_when_file_missing(
        source in prop::collection::vec("[a-z]{1,8}\\.rs", 2..10usize),
    ) {
        let mut splits = vec![source.clone()];
        // Remove the last file from splits but keep it in source
        if let Some(last) = splits[0].pop() {
            let result = verify_coverage(&source, &splits);
            prop_assert!(!result.ok);
            prop_assert!(result.orphaned.contains(&last));
        }
    }

    /// Concern classification is total — every file gets a non-empty concern.
    #[test]
    fn prop_classify_is_total(path in "[a-z/._]{1,50}") {
        let concern = classify_concern(&path);
        prop_assert!(!concern.is_empty());
    }
}

fn main() {
    println!("Run via: cargo test -p godmode-conformance decompose");
}
