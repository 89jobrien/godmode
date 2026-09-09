//! Pre-commit hook logic for task state enforcement and Cargo gates.
//!
//! Ported from `hooks/pre-commit.nu` and `hooks/scripts/pre-commit-gate.nu`.

use std::path::Path;

use anyhow::Result;

use crate::graph;
use crate::hooks::quality_gate;
use crate::model::Status;

/// Outcome of the pre-commit check.
#[derive(Debug)]
pub enum PreCommitResult {
    /// All checks passed.
    Pass,
    /// Blocked with a reason.
    Block(
        /// Explanation of the failed pre-commit check.
        String,
    ),
}

/// Run the full pre-commit sequence: task state followed by Cargo gates.
pub fn run(root: &Path) -> PreCommitResult {
    // Step 1: Check task state
    if let Err(reason) = check_task_state(root) {
        return PreCommitResult::Block(reason);
    }

    // Step 2: Cargo gates (fmt + clippy + nextest)
    if let Err(e) = quality_gate::run(root, None) {
        return PreCommitResult::Block(e.to_string());
    }

    PreCommitResult::Pass
}

/// Run only task state + lint (no tests). Used by PreToolUse/Bash gate for speed.
pub fn run_lint_gate(root: &Path) -> PreCommitResult {
    if let Err(reason) = check_task_state(root) {
        return PreCommitResult::Block(reason);
    }

    if let Err(e) = quality_gate::run_lint_only(root, None) {
        return PreCommitResult::Block(e.to_string());
    }

    PreCommitResult::Pass
}

/// Check that no tasks are in running or blocked state.
fn check_task_state(root: &Path) -> Result<(), String> {
    let task_file = crate::graph::task_file(root);
    if !task_file.exists() {
        return Ok(());
    }

    let task_graph = match graph::load(root) {
        Ok(g) => g,
        Err(_) => return Ok(()), // Degrade gracefully
    };

    // Check running tasks
    let running: Vec<&str> = task_graph
        .tasks
        .iter()
        .filter(|t| t.status == Status::Running)
        .map(|t| t.id.as_str())
        .collect();

    if !running.is_empty() {
        return Err(format!(
            "tasks still running: {}. \
             Resolve with `godmode task done <id>` or `godmode task block <id> <reason>`.",
            running.join(", ")
        ));
    }

    // Check blocked tasks
    let blocked: Vec<String> = task_graph
        .tasks
        .iter()
        .filter(|t| t.status == Status::Blocked)
        .map(|t| {
            if t.notes.is_empty() {
                t.id.clone()
            } else {
                format!("{}: {}", t.id, t.notes)
            }
        })
        .collect();

    if !blocked.is_empty() {
        return Err(format!(
            "blocked tasks must be resolved before committing:\n  - {}\n\
             Use `godmode task unblock <id>` or `godmode task remove <id>` to clear them.",
            blocked.join("\n  - ")
        ));
    }

    Ok(())
}

/// Update the legacy Claude plugin manifest with an explicit version.
///
/// This is intentionally separate from [`run`] so release-semver parity is not
/// silently replaced by a commit hash.
///
/// # Errors
///
/// Returns an error when the manifest cannot be read, parsed, or written.
///
/// # Examples
///
/// ```no_run
/// # fn main() -> anyhow::Result<()> {
/// godmode_core::hooks::pre_commit::stamp_plugin_manifest(
///     std::path::Path::new("."), "0.7.0",
/// )?;
/// # Ok(()) }
/// ```
pub fn stamp_plugin_manifest(root: &Path, version: &str) -> Result<()> {
    let path = root.join(".claude-plugin/plugin.json");
    if !path.exists() {
        return Ok(());
    }
    let raw = std::fs::read_to_string(&path)?;
    let mut value: serde_json::Value = serde_json::from_str(&raw)?;
    value["version"] = serde_json::Value::String(version.to_owned());
    std::fs::write(
        path,
        format!(
            "{}
",
            serde_json::to_string_pretty(&value)?
        ),
    )?;
    Ok(())
}

/// Format the pre-commit result for output.
pub fn format_result(result: &PreCommitResult) -> (String, i32) {
    match result {
        PreCommitResult::Pass => ("[godmode:pre-commit] all checks passed.".into(), 0),
        PreCommitResult::Block(reason) => (format!("[godmode:pre-commit] BLOCKED: {reason}"), 1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn check_task_state_ok_when_no_task_file() {
        let dir = TempDir::new().unwrap();
        assert!(check_task_state(dir.path()).is_ok());
    }

    #[test]
    fn check_task_state_ok_when_all_pending() {
        let dir = TempDir::new().unwrap();
        let ctx_dir = dir.path().join(".ctx").join("godmode");
        std::fs::create_dir_all(&ctx_dir).unwrap();
        std::fs::write(
            ctx_dir.join("tasks.yaml"),
            "tasks:\n  - id: t1\n    title: A\n    status: pending\n",
        )
        .unwrap();
        assert!(check_task_state(dir.path()).is_ok());
    }

    #[test]
    fn check_task_state_blocks_on_running() {
        let dir = TempDir::new().unwrap();
        let ctx_dir = dir.path().join(".ctx").join("godmode");
        std::fs::create_dir_all(&ctx_dir).unwrap();
        std::fs::write(
            ctx_dir.join("tasks.yaml"),
            "tasks:\n  - id: t1\n    title: A\n    status: running\n",
        )
        .unwrap();
        let result = check_task_state(dir.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("t1"));
    }

    #[test]
    fn check_task_state_blocks_on_blocked() {
        let dir = TempDir::new().unwrap();
        let ctx_dir = dir.path().join(".ctx").join("godmode");
        std::fs::create_dir_all(&ctx_dir).unwrap();
        std::fs::write(
            ctx_dir.join("tasks.yaml"),
            "tasks:\n  - id: t2\n    title: B\n    status: blocked\n    notes: broken\n",
        )
        .unwrap();
        let result = check_task_state(dir.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("t2: broken"));
    }

    #[test]
    fn format_result_pass() {
        let (msg, code) = format_result(&PreCommitResult::Pass);
        assert_eq!(code, 0);
        assert!(msg.contains("passed"));
    }

    #[test]
    fn format_result_block() {
        let (msg, code) = format_result(&PreCommitResult::Block("fmt failed".into()));
        assert_eq!(code, 1);
        assert!(msg.contains("fmt failed"));
    }
    #[test]
    fn explicit_legacy_plugin_stamp_updates_manifest_without_staging() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(".claude-plugin/plugin.json");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, serde_json::json!({"version": "old"}).to_string()).unwrap();
        stamp_plugin_manifest(dir.path(), "abc123").unwrap();
        let value: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        assert_eq!(value["version"], "abc123");
    }
}
