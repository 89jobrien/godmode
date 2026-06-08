//! cap — PostToolUse/Bash hook.
//! After git push, warns if running tasks have no commit SHA recorded.

use std::path::Path;

use crate::graph;
use crate::model::Status;

/// Run the cap hook. Returns a message for stderr (may be empty).
pub fn run(root: &Path, command: &str) -> String {
    if !command.contains("git push") {
        return String::new();
    }

    let graph = match graph::load(root) {
        Ok(g) => g,
        Err(_) => return String::new(),
    };

    let unrecorded: Vec<&str> = graph
        .tasks
        .iter()
        .filter(|t| t.status == Status::Running && t.commit.as_deref().unwrap_or("").is_empty())
        .map(|t| t.id.as_str())
        .collect();

    if unrecorded.is_empty() {
        return String::new();
    }

    let ids = unrecorded.join(", ");
    format!(
        "[godmode:cap] Push detected but running tasks have no commit — run `godmode task done <id> --commit <sha>` (tasks: {ids})"
    )
}
