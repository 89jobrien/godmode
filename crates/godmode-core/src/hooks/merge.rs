//! merge — PostToolUse/Bash hook.
//! After a successful git merge, reminds about task state sync.

use std::path::Path;

use crate::graph;
use crate::model::Status;

/// Run the merge hook. Returns a message for stderr (may be empty).
pub fn run(root: &Path, command: &str, exit_code: i64) -> String {
    if !command.contains("git merge") || exit_code != 0 {
        return String::new();
    }

    let graph = match graph::load(root) {
        Ok(g) => g,
        Err(_) => return String::new(),
    };

    let running_no_commit: Vec<&str> = graph
        .tasks
        .iter()
        .filter(|t| t.status == Status::Running && t.commit.as_deref().unwrap_or("").is_empty())
        .map(|t| t.id.as_str())
        .collect();

    if running_no_commit.is_empty() {
        return String::new();
    }

    let ids = running_no_commit.join(", ");
    format!(
        "[godmode:merge] Merge detected — mark task done: `godmode task done <id> --commit <sha>` (running tasks: {ids})"
    )
}
