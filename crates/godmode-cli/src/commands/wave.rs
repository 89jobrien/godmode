//! Wave state for parallel agent sessions.

use anyhow::Result;
use std::path::Path;

use crate::WaveAction;

pub fn run_wave_action(root: &Path, json: bool, action: WaveAction) -> Result<()> {
    match action {
        WaveAction::Init { wave, agents } => {
            let agent_refs: Vec<&str> = agents.iter().map(|s| s.as_str()).collect();
            let state = godmode_core::wave::init(root, wave, &agent_refs)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&state)?);
            } else {
                println!(
                    "Wave {} initialised: {} agent(s).",
                    wave,
                    state.agents.len()
                );
                for (name, slot) in &state.agents {
                    println!("  {} — {:?}", name, slot.status);
                }
            }
            Ok(())
        }
        WaveAction::Status => {
            let state = godmode_core::wave::load(root)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&state)?);
            } else {
                println!("Wave {}:", state.wave);
                for (name, slot) in &state.agents {
                    println!(
                        "  {:20} {:?}  commits: {}",
                        name,
                        slot.status,
                        slot.commits.join(", ")
                    );
                }
            }
            Ok(())
        }
        WaveAction::Done { agent, commits } => {
            godmode_core::wave::mark_done(root, &agent, commits)?;
            if json {
                println!(
                    "{}",
                    serde_json::json!({"ok": true, "agent": agent, "status": "done"})
                );
            } else {
                println!("Agent '{}' marked done.", agent);
            }
            Ok(())
        }
        WaveAction::Block { agent } => {
            godmode_core::wave::mark_blocked(root, &agent)?;
            if json {
                println!(
                    "{}",
                    serde_json::json!({"ok": true, "agent": agent, "status": "blocked"})
                );
            } else {
                println!("Agent '{}' marked blocked.", agent);
            }
            Ok(())
        }
        WaveAction::Check => {
            let state = godmode_core::wave::load(root)?;
            let settled = godmode_core::wave::check(&state);
            if json {
                println!(
                    "{}",
                    serde_json::json!({"settled": settled, "all_done": godmode_core::wave::all_done(&state)})
                );
            } else if settled {
                println!(
                    "Wave settled. all_done={}",
                    godmode_core::wave::all_done(&state)
                );
            } else {
                let pending: Vec<_> = state
                    .agents
                    .iter()
                    .filter(|(_, s)| s.status == godmode_core::wave::SlotStatus::Pending)
                    .map(|(n, _)| n.as_str())
                    .collect();
                println!("Wave not settled. Pending: {}", pending.join(", "));
            }
            if !settled {
                std::process::exit(1);
            }
            Ok(())
        }
    }
}
