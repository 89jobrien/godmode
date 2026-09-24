//! Parses Markdown plans and ingests their tasks into the session graph.

use std::path::Path;

use anyhow::Result;

use crate::*;

/// Reads a plan file, parses its tasks, and idempotently ingests them with source provenance.
pub fn handle(command: Cmd, root: &Path, json: bool, _sarif: bool) -> Result<()> {
    match command {
        Cmd::Plan { action } => match action {
            PlanAction::Ingest { path } => {
                let markdown = std::fs::read_to_string(&path)?;
                let mut session = Session::open(root)?;
                let parsed_tasks = plan::parse(&markdown)?;
                let source = Path::new(&path).canonicalize()?;
                let source = source.to_string_lossy();
                let report = session.ingest_plan(parsed_tasks, &source)?;
                if json {
                    println!(
                        "{}",
                        serde_json::json!({
                            "ok": true,
                            "path": path,
                            "parsed": report.parsed,
                            "added": report.added,
                            "ingested": report.added,
                            "skipped": report.skipped,
                            "ids": report.ids,
                        })
                    );
                } else {
                    println!(
                        "Ingested {} of {} tasks from {} ({} skipped).",
                        report.added, report.parsed, path, report.skipped
                    );
                }
                Ok(())
            }
        },
        _ => unreachable!("dispatcher sent command to the wrong handler"),
    }
}
