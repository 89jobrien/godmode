use std::path::Path;

use anyhow::Result;

use crate::*;

pub fn handle(command: Cmd, _root: &Path, json: bool, _sarif: bool) -> Result<()> {
    match command {
        Cmd::Issue { action } => match action {
            IssueAction::List { repo, label } => {
                let tasks =
                    godmode_core::integrations::gh::pull_issues(repo.as_deref(), label.as_deref())?;
                if tasks.is_empty() {
                    if json {
                        println!("[]");
                    } else {
                        println!("No open issues.");
                    }
                    return Ok(());
                }
                if json {
                    println!("{}", serde_json::to_string_pretty(&tasks)?);
                } else {
                    for t in &tasks {
                        println!("[{}] {}", t.id, t.title);
                    }
                }
                Ok(())
            }
            IssueAction::Close {
                number,
                repo,
                commit,
            } => {
                godmode_core::integrations::gh::issue_close(number, repo.as_deref(), &commit)?;
                if json {
                    println!("{}", serde_json::json!({"ok": true, "number": number}));
                } else {
                    println!("Issue #{} closed (commit {}).", number, commit);
                }
                Ok(())
            }
        },
        _ => unreachable!("dispatcher sent command to the wrong handler"),
    }
}
