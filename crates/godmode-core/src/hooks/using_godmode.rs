//! using-godmode — SessionStart hook.
//! If no task graph exists, prints an orientation hint.

use std::path::Path;

use crate::graph;

/// Run the using-godmode hook. Returns a message for stderr (may be empty).
pub fn run(root: &Path) -> String {
    let task_file = graph::task_file(root);
    if task_file.exists() {
        return String::new();
    }

    "[godmode] No task graph found. Run `godmode task add <title>` or `godmode plan ingest <path>` to start.".to_string()
}
