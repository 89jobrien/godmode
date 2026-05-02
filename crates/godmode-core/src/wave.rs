use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Result};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SlotStatus {
    Pending,
    Done,
    Blocked,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentSlot {
    pub status: SlotStatus,
    pub branch: String,
    pub commits: Vec<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct WaveState {
    pub wave: u32,
    pub agents: BTreeMap<String, AgentSlot>,
}

fn state_path(root: &Path) -> std::path::PathBuf {
    root.join(".ctx").join("wave-status.json")
}

pub fn init(root: &Path, wave_n: u32, agents: &[&str]) -> Result<WaveState> {
    std::fs::create_dir_all(root.join(".ctx")).context("failed to create .ctx directory")?;
    let state = WaveState {
        wave: wave_n,
        agents: agents
            .iter()
            .map(|name| {
                (
                    name.to_string(),
                    AgentSlot {
                        status: SlotStatus::Pending,
                        branch: name.to_string(),
                        commits: vec![],
                    },
                )
            })
            .collect(),
    };
    save(root, &state)?;
    Ok(state)
}

pub fn load(root: &Path) -> Result<WaveState> {
    let path = state_path(root);
    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_str(&content).context("failed to deserialise wave state")
}

pub fn save(root: &Path, state: &WaveState) -> Result<()> {
    let path = state_path(root);
    let json = serde_json::to_string_pretty(state).context("failed to serialise wave state")?;
    std::fs::write(&path, json).with_context(|| format!("failed to write {}", path.display()))
}

pub fn mark_done(root: &Path, agent: &str, commits: Vec<String>) -> Result<()> {
    let mut state = load(root)?;
    let slot = state
        .agents
        .get_mut(agent)
        .with_context(|| format!("agent '{agent}' not found in wave state"))?;
    slot.status = SlotStatus::Done;
    slot.commits = commits;
    save(root, &state)
}

pub fn mark_blocked(root: &Path, agent: &str) -> Result<()> {
    let mut state = load(root)?;
    let slot = state
        .agents
        .get_mut(agent)
        .with_context(|| format!("agent '{agent}' not found in wave state"))?;
    slot.status = SlotStatus::Blocked;
    save(root, &state)
}

/// Returns true if no slot is Pending (all settled — done or blocked).
pub fn check(state: &WaveState) -> bool {
    state
        .agents
        .values()
        .all(|s| s.status != SlotStatus::Pending)
}

/// Returns true if every slot is Done.
pub fn all_done(state: &WaveState) -> bool {
    state.agents.values().all(|s| s.status == SlotStatus::Done)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn setup(agents: &[&str]) -> (tempfile::TempDir, WaveState) {
        let dir = tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".ctx")).unwrap();
        let state = init(dir.path(), 1, agents).unwrap();
        (dir, state)
    }

    #[test]
    fn wave_init_creates_pending_slots() {
        let (_dir, state) = setup(&["alpha", "beta"]);
        assert_eq!(state.wave, 1);
        assert_eq!(state.agents.len(), 2);
        for slot in state.agents.values() {
            assert_eq!(slot.status, SlotStatus::Pending);
            assert!(slot.commits.is_empty());
        }
    }

    #[test]
    fn wave_roundtrip_save_load() {
        let (dir, state) = setup(&["alpha", "beta"]);
        save(dir.path(), &state).unwrap();
        let loaded = load(dir.path()).unwrap();
        assert_eq!(loaded.wave, state.wave);
        assert_eq!(
            loaded.agents.keys().collect::<Vec<_>>(),
            state.agents.keys().collect::<Vec<_>>()
        );
    }

    #[test]
    fn wave_mark_done_updates_slot() {
        let (dir, _) = setup(&["alpha"]);
        mark_done(dir.path(), "alpha", vec!["abc123".to_string()]).unwrap();
        let state = load(dir.path()).unwrap();
        let slot = &state.agents["alpha"];
        assert_eq!(slot.status, SlotStatus::Done);
        assert_eq!(slot.commits, vec!["abc123"]);
    }

    #[test]
    fn wave_mark_blocked_updates_slot() {
        let (dir, _) = setup(&["alpha"]);
        mark_blocked(dir.path(), "alpha").unwrap();
        let state = load(dir.path()).unwrap();
        assert_eq!(state.agents["alpha"].status, SlotStatus::Blocked);
    }

    #[test]
    fn wave_check_false_while_any_pending() {
        let (_dir, state) = setup(&["alpha", "beta"]);
        assert!(!check(&state));
    }

    #[test]
    fn wave_check_true_when_all_settled() {
        let (dir, _) = setup(&["alpha", "beta"]);
        mark_done(dir.path(), "alpha", vec![]).unwrap();
        mark_blocked(dir.path(), "beta").unwrap();
        let state = load(dir.path()).unwrap();
        assert!(check(&state));
    }

    #[test]
    fn wave_all_done_false_if_any_blocked() {
        let (dir, _) = setup(&["alpha", "beta"]);
        mark_done(dir.path(), "alpha", vec![]).unwrap();
        mark_blocked(dir.path(), "beta").unwrap();
        let state = load(dir.path()).unwrap();
        assert!(!all_done(&state));
    }
}
