//! Initializes, injects, reminds, and reports project memory-bank state.

use std::path::Path;

use anyhow::Result;

use crate::*;

/// Dispatches memory-banking subcommands to the core integration.
pub fn handle(command: Cmd, root: &Path, json: bool, _sarif: bool) -> Result<()> {
    match command {
        Cmd::MemoryBanking { action } => {
            match action {
                MemoryBankingAction::Inject => memory_banking::inject(root, json)?,
                MemoryBankingAction::Remind => memory_banking::remind(root, json)?,
                MemoryBankingAction::Init => memory_banking::init(root)?,
                MemoryBankingAction::Status => memory_banking::status(root, json)?,
            }
            Ok(())
        }
        _ => unreachable!("dispatcher sent command to the wrong handler"),
    }
}
