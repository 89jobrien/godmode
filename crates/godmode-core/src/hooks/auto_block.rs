//! Auto-block — automatically blocks the running task when tests fail.
//!
//! Ported from `hooks/scripts/post-bash-auto-block.nu`.
//! Invoked as a PostToolUse/Bash hook.

use crate::context::TaskSummary;
use crate::hooks::hook_context::HookContext;

/// Result of the auto-block check.
#[derive(Debug, PartialEq, Eq)]
pub enum AutoBlockResult {
    /// No action needed.
    NoOp,
    /// A task was blocked.
    Blocked { task_id: String, reason: String },
}

/// Check whether a test command failed and auto-block the appropriate task.
/// Does NOT mutate state — returns the action to take. Caller must invoke
/// `godmode task block` or call graph::block directly.
pub fn check(ctx: &HookContext) -> AutoBlockResult {
    if ctx.running.is_empty() {
        return AutoBlockResult::NoOp;
    }

    let cmd = ctx.command();
    let exit_code = ctx.exit_code();

    // Only act on test commands that failed
    let is_test = cmd.contains("nextest run") || cmd.contains("cargo test");
    if !is_test || exit_code == 0 {
        return AutoBlockResult::NoOp;
    }

    // Skip if --auto-done present (task run handles its own lifecycle)
    if cmd.contains("--auto-done") {
        return AutoBlockResult::NoOp;
    }

    // Match -p flag to crate_name, else pick first running task
    let target_task = match_task_to_crate(cmd, &ctx.running);
    let task_id = match target_task {
        Some(t) => t.id.clone(),
        None => return AutoBlockResult::NoOp,
    };

    // Extract failure info from stdout
    let failure_line = extract_failure_line(ctx.stdout());
    let reason = format!("{failure_line} (exit {exit_code})");

    AutoBlockResult::Blocked { task_id, reason }
}

/// Format the auto-block result as user output.
pub fn format_result(result: &AutoBlockResult) -> Option<String> {
    match result {
        AutoBlockResult::NoOp => None,
        AutoBlockResult::Blocked { task_id, reason } => {
            Some(format!("[godmode] Auto-blocked task {task_id}: {reason}"))
        }
    }
}

/// Match a `-p <crate>` flag in the command to a running task's crate_name.
fn match_task_to_crate<'a>(cmd: &str, running: &'a [TaskSummary]) -> Option<&'a TaskSummary> {
    let crate_flag = extract_crate_flag(cmd);

    if let Some(crate_name) = crate_flag {
        let matched = running
            .iter()
            .find(|t| t.crate_name.as_deref() == Some(crate_name));
        if matched.is_some() {
            return matched;
        }
    }

    running.first()
}

/// Extract the crate name from `-p <name>` in a cargo command.
fn extract_crate_flag(cmd: &str) -> Option<&str> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    for (i, part) in parts.iter().enumerate() {
        if *part == "-p" {
            return parts.get(i + 1).copied();
        }
    }
    None
}

/// Extract the first line containing FAIL or FAILED from stdout.
fn extract_failure_line(stdout: &str) -> &str {
    stdout
        .lines()
        .find(|l| l.contains("FAIL") || l.contains("FAILED"))
        .map(|l| l.trim())
        .unwrap_or("test failure")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_crate_flag_finds_p_arg() {
        assert_eq!(
            extract_crate_flag("cargo nextest run -p godmode-core"),
            Some("godmode-core")
        );
    }

    #[test]
    fn extract_crate_flag_returns_none_without_p() {
        assert_eq!(extract_crate_flag("cargo nextest run --workspace"), None);
    }

    #[test]
    fn extract_failure_line_finds_fail() {
        let stdout = "running 5 tests\ntest foo ... FAILED\ntest bar ... ok\n";
        assert_eq!(extract_failure_line(stdout), "test foo ... FAILED");
    }

    #[test]
    fn extract_failure_line_defaults_on_no_match() {
        assert_eq!(extract_failure_line("all tests passed"), "test failure");
    }

    #[test]
    fn match_task_picks_crate_match() {
        let tasks = vec![
            TaskSummary {
                id: "t1".into(),
                title: "A".into(),
                crate_name: Some("godmode-core".into()),
            },
            TaskSummary {
                id: "t2".into(),
                title: "B".into(),
                crate_name: Some("godmode-cli".into()),
            },
        ];
        let result = match_task_to_crate("cargo nextest run -p godmode-cli", &tasks);
        assert_eq!(result.unwrap().id, "t2");
    }

    #[test]
    fn match_task_falls_back_to_first() {
        let tasks = vec![TaskSummary {
            id: "t1".into(),
            title: "A".into(),
            crate_name: None,
        }];
        let result = match_task_to_crate("cargo test --workspace", &tasks);
        assert_eq!(result.unwrap().id, "t1");
    }

    #[test]
    fn check_noop_when_no_running_tasks() {
        use std::path::PathBuf;
        let ctx = HookContext {
            input: serde_json::from_str(
                r#"{"tool_input":{"command":"cargo test"},"tool_result":{"exit_code":1}}"#,
            )
            .unwrap(),
            git_root: PathBuf::from("/tmp"),
            project: "test".into(),
            running: vec![],
            pending_count: 0,
            blocked: vec![],
            recent_commits: vec![],
        };
        assert_eq!(check(&ctx), AutoBlockResult::NoOp);
    }

    #[test]
    fn check_noop_when_exit_zero() {
        use std::path::PathBuf;
        let ctx = HookContext {
            input: serde_json::from_str(
                r#"{"tool_input":{"command":"cargo test"},"tool_result":{"exit_code":0}}"#,
            )
            .unwrap(),
            git_root: PathBuf::from("/tmp"),
            project: "test".into(),
            running: vec![TaskSummary {
                id: "t1".into(),
                title: "A".into(),
                crate_name: None,
            }],
            pending_count: 0,
            blocked: vec![],
            recent_commits: vec![],
        };
        assert_eq!(check(&ctx), AutoBlockResult::NoOp);
    }

    #[test]
    fn check_blocks_on_test_failure() {
        use std::path::PathBuf;
        let ctx = HookContext {
            input: serde_json::from_str(
                r#"{"tool_input":{"command":"cargo nextest run -p foo"},"tool_result":{"exit_code":1,"stdout":"test bar ... FAILED\n"}}"#,
            )
            .unwrap(),
            git_root: PathBuf::from("/tmp"),
            project: "test".into(),
            running: vec![TaskSummary {
                id: "t1".into(),
                title: "A".into(),
                crate_name: Some("foo".into()),
            }],
            pending_count: 0,
            blocked: vec![],
            recent_commits: vec![],
        };
        let result = check(&ctx);
        assert_eq!(
            result,
            AutoBlockResult::Blocked {
                task_id: "t1".into(),
                reason: "test bar ... FAILED (exit 1)".into(),
            }
        );
    }

    #[test]
    fn check_skips_auto_done_commands() {
        use std::path::PathBuf;
        let ctx = HookContext {
            input: serde_json::from_str(
                r#"{"tool_input":{"command":"cargo nextest run --auto-done"},"tool_result":{"exit_code":1}}"#,
            )
            .unwrap(),
            git_root: PathBuf::from("/tmp"),
            project: "test".into(),
            running: vec![TaskSummary {
                id: "t1".into(),
                title: "A".into(),
                crate_name: None,
            }],
            pending_count: 0,
            blocked: vec![],
            recent_commits: vec![],
        };
        assert_eq!(check(&ctx), AutoBlockResult::NoOp);
    }
}
