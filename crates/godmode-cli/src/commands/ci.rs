use std::path::Path;

use anyhow::Result;

use crate::*;

pub fn handle(command: Cmd, _root: &Path, json: bool, _sarif: bool) -> Result<()> {
    match command {
        Cmd::Ci { action } => match action {
            CiAction::Triage { run_id } => {
                let result = godmode_core::integrations::gh::ci_triage(run_id.as_deref())?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&result)?);
                } else {
                    println!("Run:   {}", result.run_id);
                    println!("Class: {:?}", result.class);
                    println!("Fix:   {}", result.fix_hint);
                    if !result.raw_snippet.is_empty() {
                        println!("\n--- log snippet ---\n{}", result.raw_snippet);
                    }
                }
                Ok(())
            }
        },
        _ => unreachable!("dispatcher sent command to the wrong handler"),
    }
}
