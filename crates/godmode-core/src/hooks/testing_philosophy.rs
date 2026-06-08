//! testing-philosophy — PreToolUse/Write hook.
//! Warns if a src/ file has no associated tests.

use std::path::Path;

use crate::test_check;

/// Run the testing-philosophy hook. Returns a message for stderr (may be empty).
pub fn run(root: &Path, file_path: &str) -> String {
    test_check::check_test_coverage(file_path, root).unwrap_or_default()
}
