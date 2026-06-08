//! wave-integration — PostToolUse/Bash hook.
//! After `godmode wave done` or `godmode wave check`, prints wave status summary.

use std::path::Path;

use crate::wave;

/// Run the wave-integration hook. Returns a message for stderr (may be empty).
pub fn run(root: &Path, command: &str) -> String {
    let relevant = command.contains("godmode wave done") || command.contains("godmode wave check");
    if !relevant {
        return String::new();
    }

    let state = match wave::load(root) {
        Ok(s) => s,
        Err(_) => return String::new(),
    };

    let done = state
        .agents
        .values()
        .filter(|s| s.status == wave::SlotStatus::Done)
        .count();
    let pending = state
        .agents
        .values()
        .filter(|s| s.status == wave::SlotStatus::Pending)
        .count();
    let blocked = state
        .agents
        .values()
        .filter(|s| s.status == wave::SlotStatus::Blocked)
        .count();

    let mut msg = format!(
        "[godmode:wave] Wave {}: {done} done / {pending} pending / {blocked} blocked",
        state.wave
    );

    if pending == 0 && blocked == 0 {
        msg.push_str("\n  All agents settled — ready for integration");
    }

    msg
}
