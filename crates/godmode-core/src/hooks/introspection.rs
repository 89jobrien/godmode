//! introspection — Stop hook.
//! Warns if plugin conformance issues are detected.

use std::path::Path;

use crate::review;

/// Run the introspection hook. Returns a message for stderr (may be empty).
pub fn run(root: &Path) -> String {
    let report = match review::run_all(root) {
        Ok(r) => r,
        Err(_) => return String::new(),
    };

    if report.findings.is_empty() {
        return String::new();
    }

    let issues: Vec<String> = report
        .findings
        .iter()
        .take(5)
        .map(|f| format!("  - {}", f.message))
        .collect();

    format!(
        "[godmode:introspection] Plugin conformance issues:\n{}",
        issues.join("\n")
    )
}
