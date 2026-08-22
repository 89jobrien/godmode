//! Git worktree lifecycle.

use anyhow::Result;
use std::path::Path;

use crate::WorktreeAction;

pub fn run_worktree_action(root: &Path, json: bool, action: WorktreeAction) -> Result<()> {
    match action {
        WorktreeAction::Add { branch, issue } => {
            let info = godmode_core::worktree::add(root, &branch, issue)?;
            if json {
                println!(
                    "{}",
                    serde_json::json!({"ok": true, "branch": info.branch, "path": info.path.display().to_string()})
                );
            } else {
                println!(
                    "Worktree created: {} → {}",
                    info.branch,
                    info.path.display()
                );
            }
            Ok(())
        }
        WorktreeAction::Remove { branch } => {
            godmode_core::worktree::remove(root, &branch)?;
            if json {
                println!(
                    "{}",
                    serde_json::json!({"ok": true, "branch": branch, "removed": true})
                );
            } else {
                println!("Worktree removed: {}", branch);
            }
            Ok(())
        }
    }
}
