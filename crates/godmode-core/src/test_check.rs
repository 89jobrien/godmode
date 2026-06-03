//! Test coverage check — verifies that a Rust source file has associated tests.
//!
//! Replaces `skills/testing-philosophy/hook.nu`. Returns a diagnostic message
//! if no tests are found, or `None` if the file is covered.

use std::path::Path;

/// Check whether a Rust source file has associated tests.
///
/// Returns `Some(message)` if no tests found, `None` if covered or not applicable.
pub fn check_test_coverage(file_path: &str, git_root: &Path) -> Option<String> {
    // Only care about src/ files
    if !file_path.contains("/src/") {
        return None;
    }

    // Skip entry points
    if file_path.ends_with("src/lib.rs") || file_path.ends_with("src/main.rs") {
        return None;
    }

    // Must be a Rust source file
    if !file_path.ends_with(".rs") {
        return None;
    }

    let path = Path::new(file_path);

    // Check for inline tests in the target file itself
    if path.exists()
        && std::fs::read_to_string(path).is_ok_and(|contents| contents.contains("#[cfg(test)]"))
    {
        return None;
    }

    // Derive the base name for test lookup (e.g. src/foo/bar.rs -> bar)
    let base_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

    if base_name.is_empty() {
        return None;
    }

    // Check for test files in standard locations
    let test_dirs = [
        git_root.join("tests"),
        git_root.join("tests/conformance/src"),
        git_root.join("crates/godmode-core/tests"),
    ];

    for dir in &test_dirs {
        if !dir.exists() {
            continue;
        }
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                if entry
                    .file_name()
                    .to_str()
                    .is_some_and(|name| name.contains(base_name))
                {
                    return None;
                }
            }
        }
    }

    Some(format!(
        "[godmode:testing-philosophy] No tests for {file_path} \
         — consult /godmode:testing-philosophy"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn non_src_file_returns_none() {
        let tmp = tempfile::tempdir().expect("tempdir");
        assert!(check_test_coverage("docs/readme.md", tmp.path()).is_none());
    }

    #[test]
    fn lib_rs_returns_none() {
        let tmp = tempfile::tempdir().expect("tempdir");
        assert!(check_test_coverage("crates/foo/src/lib.rs", tmp.path()).is_none());
    }

    #[test]
    fn main_rs_returns_none() {
        let tmp = tempfile::tempdir().expect("tempdir");
        assert!(check_test_coverage("crates/foo/src/main.rs", tmp.path()).is_none());
    }

    #[test]
    fn non_rs_file_returns_none() {
        let tmp = tempfile::tempdir().expect("tempdir");
        assert!(check_test_coverage("crates/foo/src/build.py", tmp.path()).is_none());
    }

    #[test]
    fn file_with_inline_tests_returns_none() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let src_dir = tmp.path().join("crates/foo/src");
        fs::create_dir_all(&src_dir).expect("mkdir");
        let file_path = src_dir.join("bar.rs");
        fs::write(&file_path, "fn bar() {}\n#[cfg(test)]\nmod tests {}\n").expect("write");

        assert!(check_test_coverage(file_path.to_str().expect("path"), tmp.path()).is_none());
    }

    #[test]
    fn file_without_tests_returns_warning() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let src_dir = tmp.path().join("crates/foo/src");
        fs::create_dir_all(&src_dir).expect("mkdir");
        let file_path = src_dir.join("bar.rs");
        fs::write(&file_path, "fn bar() {}\n").expect("write");

        let result = check_test_coverage(file_path.to_str().expect("path"), tmp.path());
        assert!(result.is_some());
        assert!(result.unwrap().contains("No tests for"));
    }

    #[test]
    fn matching_test_file_returns_none() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let src_dir = tmp.path().join("crates/foo/src");
        let test_dir = tmp.path().join("tests");
        fs::create_dir_all(&src_dir).expect("mkdir src");
        fs::create_dir_all(&test_dir).expect("mkdir tests");

        let file_path = src_dir.join("graph.rs");
        fs::write(&file_path, "fn graph() {}\n").expect("write src");
        fs::write(test_dir.join("graph_integration.rs"), "").expect("write test");

        assert!(check_test_coverage(file_path.to_str().expect("path"), tmp.path()).is_none());
    }
}
