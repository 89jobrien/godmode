//! Pre-commit hook logic — task state enforcement + plugin.json stamp + cargo gates.
//!
//! Ported from `hooks/pre-commit.nu` and `hooks/scripts/pre-commit-gate.nu`.

use std::path::Path;
use std::process::Command;

use anyhow::{Result, bail};

use crate::graph;
use crate::hooks::quality_gate;
use crate::model::Status;

/// Outcome of the pre-commit check.
#[derive(Debug)]
pub enum PreCommitResult {
    /// All checks passed.
    Pass,
    /// Blocked with a reason.
    Block(String),
}

/// Run the full pre-commit sequence: task state, plugin stamp, cargo gates.
pub fn run(root: &Path) -> PreCommitResult {
    // Step 1: Check task state
    if let Err(reason) = check_task_state(root) {
        return PreCommitResult::Block(reason);
    }

    // Step 2: Stamp plugin.json version
    if let Err(e) = stamp_plugin_version(root) {
        eprintln!("[godmode:pre-commit] plugin stamp warning: {e}");
        // Non-fatal — continue
    }

    // Step 3: Cargo gates (fmt + clippy + nextest)
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
    let task_file = root.join(".ctx/GODMODE.tasks.yaml");
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

/// Stamp `.claude-plugin/plugin.json` version with current HEAD short hash.
fn stamp_plugin_version(root: &Path) -> Result<()> {
    let plugin_path = root.join(".claude-plugin/plugin.json");
    if !plugin_path.exists() {
        return Ok(());
    }

    let short_hash = {
        let out = Command::new("git")
            .args(["rev-parse", "--short", "HEAD"])
            .current_dir(root)
            .output()?;
        if !out.status.success() {
            bail!("git rev-parse failed");
        }
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    };

    let content = std::fs::read_to_string(&plugin_path)?;
    let mut json: serde_json::Value = serde_json::from_str(&content)?;

    let current_version = json.get("version").and_then(|v| v.as_str()).unwrap_or("");

    if current_version != short_hash {
        json["version"] = serde_json::Value::String(short_hash.clone());
        let new_content = serde_json::to_string_pretty(&json)?;
        std::fs::write(&plugin_path, format!("{new_content}\n"))?;

        // Stage the updated file
        Command::new("git")
            .args(["add", &plugin_path.to_string_lossy()])
            .current_dir(root)
            .output()?;

        eprintln!("[godmode:pre-commit] stamped plugin.json version -> {short_hash}");
    }

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
        let ctx_dir = dir.path().join(".ctx");
        std::fs::create_dir_all(&ctx_dir).unwrap();
        std::fs::write(
            ctx_dir.join("GODMODE.tasks.yaml"),
            "tasks:\n  - id: t1\n    title: A\n    status: pending\n",
        )
        .unwrap();
        assert!(check_task_state(dir.path()).is_ok());
    }

    #[test]
    fn check_task_state_blocks_on_running() {
        let dir = TempDir::new().unwrap();
        let ctx_dir = dir.path().join(".ctx");
        std::fs::create_dir_all(&ctx_dir).unwrap();
        std::fs::write(
            ctx_dir.join("GODMODE.tasks.yaml"),
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
        let ctx_dir = dir.path().join(".ctx");
        std::fs::create_dir_all(&ctx_dir).unwrap();
        std::fs::write(
            ctx_dir.join("GODMODE.tasks.yaml"),
            "tasks:\n  - id: t2\n    title: B\n    status: blocked\n    notes: broken\n",
        )
        .unwrap();
        let result = check_task_state(dir.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("t2: broken"));
    }

    #[test]
    fn stamp_plugin_version_noop_when_no_plugin_json() {
        let dir = TempDir::new().unwrap();
        assert!(stamp_plugin_version(dir.path()).is_ok());
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
}
