//! Produces the session-end handoff summary.

use std::path::Path;

use anyhow::Result;

use crate::*;

/// Runs handoff integrations and prints the resulting session summary.
pub fn handle(command: Cmd, root: &Path, json: bool, _sarif: bool) -> Result<()> {
    match command {
        Cmd::Handoff => {
            let out = integrations::handoff(root)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&out)?);
            } else {
                print!("{}", out.human);
            }
            Ok(())
        }
        _ => unreachable!("dispatcher sent command to the wrong handler"),
    }
}
