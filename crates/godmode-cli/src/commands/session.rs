//! `handon`, `handoff`, and session-file maintenance.

use anyhow::Result;
use godmode_core::integrations;
use godmode_core::integrations::crux;
use godmode_core::session::prune_sessions_older_than;
use std::path::Path;

use crate::SessionAction;

pub fn run_handon(root: &Path, json: bool, compact: bool) -> Result<()> {
    let out = integrations::handon(root)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else if compact {
        let g = &out.graph;
        println!(
            "godmode: {}D {}R {}P {}B",
            g.done, g.running, g.pending, g.blocked
        );
    } else {
        print!("{}", out.human);
    }
    Ok(())
}

pub fn run_handoff(root: &Path, json: bool) -> Result<()> {
    let out = integrations::handoff(root)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        print!("{}", out.human);
    }
    Ok(())
}

pub fn run_session_action(root: &Path, json: bool, action: SessionAction) -> Result<()> {
    match action {
        SessionAction::Prune {
            older_than,
            dry_run,
        } => {
            let sessions_dir = crux::sessions_dir(root);
            let pruned = prune_sessions_older_than(&sessions_dir, older_than, dry_run)?;
            if json {
                let paths: Vec<String> = pruned.iter().map(|p| p.display().to_string()).collect();
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
    }
}
