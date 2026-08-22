//! CI failure triage.

use anyhow::Result;

use crate::CiAction;

pub fn run_ci_action(json: bool, action: CiAction) -> Result<()> {
    match action {
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
    }
}
