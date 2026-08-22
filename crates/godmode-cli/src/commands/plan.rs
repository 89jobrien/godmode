//! Plan markdown ingestion.

use anyhow::Result;
use godmode_core::plan;
use godmode_core::session::Session;
use std::path::Path;

use crate::PlanAction;

pub fn run_plan_action(root: &Path, json: bool, action: PlanAction) -> Result<()> {
    match action {
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
    }
}
