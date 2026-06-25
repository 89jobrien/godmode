//! cap — PostToolUse/Bash hook.
//! After git push, warns if running tasks have no commit SHA recorded.

use std::path::Path;

use super::hook_context::running_without_commit_ids;

/// Run the cap hook. Returns a message for stderr (may be empty).
pub fn run(root: &Path, command: &str) -> String {
    if !command.contains("git push") {
        return String::new();
    }
    let Some(ids) = running_without_commit_ids(root) else {
        return String::new();
    };
    format!(
        "[godmode:cap] Push detected but running tasks have no commit — \
         run `godmode task done <id> --commit <sha>` (tasks: {ids})"
    )
}
