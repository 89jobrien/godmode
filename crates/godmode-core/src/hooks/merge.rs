//! merge — PostToolUse/Bash hook.
//! After a successful git merge, reminds about task state sync.

use std::path::Path;

use super::hook_context::running_without_commit_ids;

/// Run the merge hook. Returns a message for stderr (may be empty).
pub fn run(root: &Path, command: &str, exit_code: i64) -> String {
    if !command.contains("git merge") || exit_code != 0 {
        return String::new();
    }
    let Some(ids) = running_without_commit_ids(root) else {
        return String::new();
    };
    format!(
        "[godmode:merge] Merge detected — mark task done: \
         `godmode task done <id> --commit <sha>` (running tasks: {ids})"
    )
}
