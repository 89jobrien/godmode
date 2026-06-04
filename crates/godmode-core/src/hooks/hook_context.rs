//! Shared hook context — Rust equivalent of `godmode-hook-lib.nu`.
//!
//! Assembles the same context record that `godmode context --json` emits,
//! but without spawning a subprocess. Hooks call `HookContext::load()` at
//! the top; if any precondition fails, they get `None` and exit 0.

use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::Value;

use crate::context::{self, BlockedSummary, SessionContext, TaskSummary};

/// Parsed hook stdin JSON (tool_input + tool_result fields vary by hook type).
#[derive(Debug, Clone, Deserialize)]
pub struct HookInput {
    #[serde(default)]
    pub tool_input: Value,
    #[serde(default)]
    pub tool_result: Value,
}

/// Full context available to every hook.
#[derive(Debug)]
pub struct HookContext {
    pub input: HookInput,
    pub git_root: PathBuf,
    pub project: String,
    pub running: Vec<TaskSummary>,
    pub pending_count: usize,
    pub blocked: Vec<BlockedSummary>,
    pub recent_commits: Vec<String>,
}

impl HookContext {
    /// Build hook context from stdin JSON and the current repo.
    /// Returns `None` if any precondition fails (not a git repo, no task file, etc.).
    pub fn load(stdin_json: &str) -> Option<Self> {
        let input: HookInput = serde_json::from_str(stdin_json).ok()?;
        let git_root = find_git_root()?;
        let task_file = crate::graph::task_file(&git_root);
        if !task_file.exists() {
            return None;
        }
        let ctx = context::build(&git_root).ok()?;
        Some(Self::from_session_context(input, git_root, ctx))
    }

    /// Build from an already-resolved git root (skips git-root detection).
    pub fn load_with_root(stdin_json: &str, git_root: &Path) -> Option<Self> {
        let input: HookInput = serde_json::from_str(stdin_json).ok()?;
        let task_file = crate::graph::task_file(git_root);
        if !task_file.exists() {
            return None;
        }
        let ctx = context::build(git_root).ok()?;
        Some(Self::from_session_context(
            input,
            git_root.to_path_buf(),
            ctx,
        ))
    }

    fn from_session_context(input: HookInput, git_root: PathBuf, ctx: SessionContext) -> Self {
        Self {
            input,
            git_root,
            project: ctx.project,
            running: ctx.running,
            pending_count: ctx.pending_count,
            blocked: ctx.blocked,
            recent_commits: ctx.recent_commits,
        }
    }

    /// Extract `tool_input.command` string (for Bash hooks).
    pub fn command(&self) -> &str {
        self.input
            .tool_input
            .get("command")
            .and_then(|v| v.as_str())
            .unwrap_or("")
    }

    /// Extract `tool_result.exit_code` (for PostToolUse hooks).
    pub fn exit_code(&self) -> i64 {
        self.input
            .tool_result
            .get("exit_code")
            .and_then(|v| v.as_i64())
            .unwrap_or(0)
    }

    /// Extract `tool_result.stdout` (for PostToolUse hooks).
    pub fn stdout(&self) -> &str {
        self.input
            .tool_result
            .get("stdout")
            .and_then(|v| v.as_str())
            .unwrap_or("")
    }

    /// Extract `tool_input.file_path` (for Write/Edit hooks).
    pub fn file_path(&self) -> &str {
        self.input
            .tool_input
            .get("file_path")
            .and_then(|v| v.as_str())
            .unwrap_or("")
    }
}

/// Walk up from CWD to find the git root.
fn find_git_root() -> Option<PathBuf> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if root.is_empty() {
        return None;
    }
    Some(PathBuf::from(root))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_returns_none_on_invalid_json() {
        assert!(HookContext::load("not json").is_none());
    }

    #[test]
    fn hook_input_parses_minimal_json() {
        let json = r#"{"tool_input":{"command":"cargo test"},"tool_result":{"exit_code":1}}"#;
        let input: HookInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.tool_input["command"], "cargo test");
        assert_eq!(input.tool_result["exit_code"], 1);
    }

    #[test]
    fn hook_input_handles_empty_json() {
        let json = "{}";
        let input: HookInput = serde_json::from_str(json).unwrap();
        assert!(input.tool_input.is_null());
        assert!(input.tool_result.is_null());
    }

    #[test]
    fn command_extracts_from_tool_input() {
        let json = r#"{"tool_input":{"command":"cargo nextest run -p foo"}}"#;
        let input: HookInput = serde_json::from_str(json).unwrap();
        let ctx = HookContext {
            input,
            git_root: PathBuf::from("/tmp"),
            project: "test".into(),
            running: vec![],
            pending_count: 0,
            blocked: vec![],
            recent_commits: vec![],
        };
        assert_eq!(ctx.command(), "cargo nextest run -p foo");
    }

    #[test]
    fn exit_code_defaults_to_zero() {
        let json = r#"{}"#;
        let input: HookInput = serde_json::from_str(json).unwrap();
        let ctx = HookContext {
            input,
            git_root: PathBuf::from("/tmp"),
            project: "test".into(),
            running: vec![],
            pending_count: 0,
            blocked: vec![],
            recent_commits: vec![],
        };
        assert_eq!(ctx.exit_code(), 0);
    }
}
