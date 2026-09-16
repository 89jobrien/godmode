use std::path::Path;

use anyhow::Result;

use crate::*;

pub fn handle(command: Cmd, root: &Path, json: bool, _sarif: bool) -> Result<()> {
    match command {
        Cmd::Worktree { action } => match action {
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
        },
        _ => unreachable!("dispatcher sent command to the wrong handler"),
    }
}
