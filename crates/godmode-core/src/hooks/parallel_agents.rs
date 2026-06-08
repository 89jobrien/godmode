//! parallel-agents — PostToolUse/Agent hook.
//! Warns when running tasks have no commit recorded.

use std::path::Path;

use crate::graph;
use crate::model::Status;

/// Run the parallel-agents hook. Returns a message for stderr (may be empty).
pub fn run(root: &Path) -> String {
    let graph = match graph::load(root) {
        Ok(g) => g,
        Err(_) => return String::new(),
    };

    let orphans: Vec<_> = graph
        .tasks
        .iter()
        .filter(|t| t.status == Status::Running && t.commit.as_deref().unwrap_or("").is_empty())
        .collect();

    if orphans.is_empty() {
        return String::new();
    }

    "[godmode:parallel-agents] Running tasks detected with no commit — verify subagents committed their work: `godmode task list`".to_string()
}
