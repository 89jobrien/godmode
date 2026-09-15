use std::path::Path;

use anyhow::Result;

use crate::*;

pub fn handle(command: Cmd, root: &Path, json: bool, _sarif: bool) -> Result<()> {
    match command {
        Cmd::Handon { compact } => {
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
        _ => unreachable!("dispatcher sent command to the wrong handler"),
    }
}
