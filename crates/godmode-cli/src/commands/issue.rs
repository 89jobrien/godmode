use std::path::Path;

use anyhow::Result;

use crate::*;

pub fn handle(command: Cmd, root: &Path, json: bool, _sarif: bool) -> Result<()> {
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
            IssueAction::SyncTodos {
                repo,
                preview,
                apply,
            } => {
                let client = godmode_core::integrations::gh::GhIssueClient::new(repo.as_deref())?;
                let mode = if apply && !preview {
                    godmode_core::write_mode::WriteMode::Apply
                } else {
                    godmode_core::write_mode::WriteMode::Preview
                };
                let result = godmode_core::todo_issue_sync::sync(root, &client, mode)?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&result)?);
                } else {
                    println!(
                        "TODO sync ({}): {} total, {} covered, {} missing, {} created",
                        result.mode, result.total, result.covered, result.missing, result.created
                    );
                    for item in &result.items {
                        println!(
                            "{}:{}  {}  {}",
                            item.path, item.line, item.fingerprint, item.text
                        );
                    }
                    if !apply && result.missing > 0 {
                        println!("Re-run with --apply to create missing issues.");
                    }
                }
                Ok(())
            }
        },
        _ => unreachable!("dispatcher sent command to the wrong handler"),
    }
}
