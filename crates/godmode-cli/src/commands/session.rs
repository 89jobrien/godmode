use std::path::Path;

use anyhow::Result;

use crate::*;

pub fn handle(command: Cmd, root: &Path, json: bool, _sarif: bool) -> Result<()> {
    match command {
        Cmd::Session { action } => match action {
            SessionAction::Prune {
                older_than,
                dry_run,
            } => {
                use godmode_core::integrations::crux;
                use godmode_core::session::prune_sessions_older_than;
                let sessions_dir = crux::sessions_dir(root);
                let pruned = prune_sessions_older_than(&sessions_dir, older_than, dry_run)?;
                if json {
                    let paths: Vec<String> =
                        pruned.iter().map(|p| p.display().to_string()).collect();
                    println!("{}", serde_json::to_string_pretty(&paths)?);
                } else if pruned.is_empty() {
                    println!("No session files to prune.");
                } else if dry_run {
                    println!("{} file(s) would be deleted.", pruned.len());
                } else {
                    println!("Pruned {} session file(s).", pruned.len());
                }
                Ok(())
            }
        },
        _ => unreachable!("dispatcher sent command to the wrong handler"),
    }
}
