//! refactoring — PreToolUse/Edit hook.
//! Warns if editing without a running task in the graph.

use std::path::Path;

use crate::graph;
use crate::model::Status;

/// Run the refactoring hook. Returns a message for stderr (may be empty).
pub fn run(root: &Path) -> String {
    let g = match graph::load(root) {
        Ok(g) => g,
        Err(_) => return String::new(),
    };

    let running = g.tasks.iter().any(|t| t.status == Status::Running);
    if running {
        String::new()
    } else {
        "[godmode:refactoring] No task running during edit — start a task before refactoring: `godmode task start <id>`".to_string()
    }
}
