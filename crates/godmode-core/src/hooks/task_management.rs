//! task-management — SessionStart hook.
//! Prints a one-line task status summary.

use std::path::Path;

use crate::graph;

/// Run the task-management hook. Returns a message for stderr (may be empty).
pub fn run(root: &Path) -> String {
    let graph = match graph::load(root) {
        Ok(g) => g,
        Err(_) => return String::new(),
    };

    let tasks = &graph.tasks;
    if tasks.is_empty() {
        return String::new();
    }

    let done = tasks
        .iter()
        .filter(|t| t.status == crate::model::Status::Done)
        .count();
    let running = tasks
        .iter()
        .filter(|t| t.status == crate::model::Status::Running)
        .count();
    let pending = tasks
        .iter()
        .filter(|t| t.status == crate::model::Status::Pending)
        .count();
    let blocked = tasks
        .iter()
        .filter(|t| t.status == crate::model::Status::Blocked)
        .count();

    let mut msg = format!(
        "[godmode] {done} done / {running} running / {pending} pending / {blocked} blocked"
    );

    if blocked > 0 {
        msg.push_str(
            "\n  blocked: run `godmode task unblock-all` or `godmode task list` to review",
        );
    }

    msg
}
