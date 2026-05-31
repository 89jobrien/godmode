//! Stop guard — blocks session end if tasks are still running or blocked.
//!
//! Ported from `hooks/scripts/stop-guard.nu`.

use std::path::Path;

use crate::context;

/// Outcome of the stop guard check.
#[derive(Debug, PartialEq, Eq)]
pub enum StopDecision {
    /// Session may end.
    Allow,
    /// Session blocked — running tasks exist.
    BlockedRunning(Vec<String>),
    /// Session blocked — blocked tasks need resolution.
    BlockedTasks(Vec<(String, String)>),
}

/// Check whether the session is safe to end.
/// Returns `Allow` if no task file exists or no problematic tasks found.
pub fn check(root: &Path) -> StopDecision {
    let init_file = root.join(".ctx/.initialized");
    if !init_file.exists() {
        return StopDecision::Allow;
    }

    let ctx = match context::build(root) {
        Ok(ctx) => ctx,
        Err(_) => return StopDecision::Allow,
    };

    if !ctx.running.is_empty() {
        let ids: Vec<String> = ctx.running.iter().map(|t| t.id.clone()).collect();
        return StopDecision::BlockedRunning(ids);
    }

    if !ctx.blocked.is_empty() {
        let items: Vec<(String, String)> = ctx
            .blocked
            .iter()
            .map(|t| (t.id.clone(), t.reason.clone()))
            .collect();
        return StopDecision::BlockedTasks(items);
    }

    StopDecision::Allow
}

/// Format the stop decision as user-facing output. Returns (message, exit_code).
pub fn format_decision(decision: &StopDecision) -> (String, i32) {
    match decision {
        StopDecision::Allow => (String::new(), 0),
        StopDecision::BlockedRunning(ids) => {
            let id_list = ids.join(", ");
            let msg = format!(
                "[godmode] Session blocked: tasks still running: {id_list}\n\
                 Mark them done or blocked before ending the session:\n  \
                 godmode task done <id> --commit <sha>\n  \
                 godmode task block <id> <reason>"
            );
            (msg, 1)
        }
        StopDecision::BlockedTasks(items) => {
            let lines: Vec<String> = items
                .iter()
                .map(|(id, reason)| {
                    if reason.is_empty() {
                        format!("  - {id}")
                    } else {
                        format!("  - {id}: {reason}")
                    }
                })
                .collect();
            let msg = format!(
                "[godmode] Session blocked: blocked tasks must be resolved:\n{}\n\
                 Use `godmode task unblock <id>` or `godmode task remove <id>` to clear them.",
                lines.join("\n")
            );
            (msg, 1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn allow_when_no_init_file() {
        let dir = TempDir::new().unwrap();
        assert_eq!(check(dir.path()), StopDecision::Allow);
    }

    #[test]
    fn allow_when_no_task_file() {
        let dir = TempDir::new().unwrap();
        let ctx_dir = dir.path().join(".ctx");
        std::fs::create_dir_all(&ctx_dir).unwrap();
        std::fs::write(ctx_dir.join(".initialized"), "").unwrap();
        // No GODMODE.tasks.yaml — context::build will fail gracefully
        assert_eq!(check(dir.path()), StopDecision::Allow);
    }

    #[test]
    fn format_allow_is_empty() {
        let (msg, code) = format_decision(&StopDecision::Allow);
        assert!(msg.is_empty());
        assert_eq!(code, 0);
    }

    #[test]
    fn format_blocked_running() {
        let decision = StopDecision::BlockedRunning(vec!["t1".into(), "t2".into()]);
        let (msg, code) = format_decision(&decision);
        assert_eq!(code, 1);
        assert!(msg.contains("t1, t2"));
        assert!(msg.contains("godmode task done"));
    }

    #[test]
    fn format_blocked_tasks() {
        let decision = StopDecision::BlockedTasks(vec![("t3".into(), "test failure".into())]);
        let (msg, code) = format_decision(&decision);
        assert_eq!(code, 1);
        assert!(msg.contains("t3: test failure"));
        assert!(msg.contains("godmode task unblock"));
    }
}
