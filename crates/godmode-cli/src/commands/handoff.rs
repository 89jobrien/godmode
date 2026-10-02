//! Produces the session-end handoff summary.

use std::path::Path;

use anyhow::Result;

use crate::*;

/// Runs handoff integrations and prints the resulting session summary.
pub fn handle(command: Cmd, root: &Path, json: bool, _sarif: bool) -> Result<()> {
    match command {
        Cmd::Handoff => {
            let out = integrations::handoff(root)?;
            // Close the session in the trace. Pairs with the `session.start`
            // emitted by `handon`, so every session is bracketed even when it
            // invoked no instrumented helper.
            godmode_core::hooks::trace_log::append(
                root,
                "session.end",
                serde_json::json!({
                    "graph": {
                        "done": out.graph.done,
                        "running": out.graph.running,
                        "pending": out.graph.pending,
                        "blocked": out.graph.blocked,
                    },
                    "dirty_files": out.dirty_files.len(),
                }),
            );
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
