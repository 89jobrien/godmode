//! Produces the session-start task and integration summary.

use std::path::Path;

use anyhow::Result;

use crate::*;

/// Runs handon integrations and prints full or compact graph state.
pub fn handle(command: Cmd, root: &Path, json: bool, _sarif: bool) -> Result<()> {
    match command {
        Cmd::Handon { compact } => {
            let out = integrations::handon(root)?;
            // Bracket the session in the trace. Helper-level tracing is opt-in, so
            // without this a session that invoked no instrumented helper left no
            // record at all and `trace-stats.nu` reported a stale log.
            godmode_core::hooks::trace_log::append(
                root,
                "session.start",
                serde_json::json!({
                    "graph": {
                        "done": out.graph.done,
                        "running": out.graph.running,
                        "pending": out.graph.pending,
                        "blocked": out.graph.blocked,
                    }
                }),
            );
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
