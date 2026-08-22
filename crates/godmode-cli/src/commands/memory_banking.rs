//! Persistent source-backed project context.

use anyhow::Result;
use godmode_core::memory_banking;
use std::path::Path;

use crate::MemoryBankingAction;

pub fn run_memory_banking_action(
    root: &Path,
    json: bool,
    action: MemoryBankingAction,
) -> Result<()> {
    match action {
        MemoryBankingAction::Inject => memory_banking::inject(root, json)?,
        MemoryBankingAction::Remind => memory_banking::remind(root, json)?,
        MemoryBankingAction::Init => memory_banking::init(root)?,
        MemoryBankingAction::Status => memory_banking::status(root, json)?,
    }
    Ok(())
}
