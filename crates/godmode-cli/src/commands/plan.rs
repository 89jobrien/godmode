use std::path::Path;

use anyhow::Result;

use crate::*;

pub fn handle(command: Cmd, root: &Path, json: bool, _sarif: bool) -> Result<()> {
    match command {
        Cmd::Plan { action } => match action {
            PlanAction::Ingest { path } => {
                let markdown = std::fs::read_to_string(&path)?;
                let tasks = plan::parse(&markdown)?;
                let count = tasks.len();
                let mut session = Session::open(root)?;
                for task in tasks {
                    if let Err(e) = session.add_task(task)
                        && !e.to_string().contains("already exists")
                    {
                        return Err(e);
                    }
                }
                session.save()?;
                if json {
                    println!("{}", serde_json::json!({"ok": true, "ingested": count}));
                } else {
                    println!("Ingested {} tasks from {}.", count, path);
                }
                Ok(())
            }
        },
        _ => unreachable!("dispatcher sent command to the wrong handler"),
    }
}
