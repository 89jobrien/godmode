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
    root.join(".ctx").join("godmode").join("wave-status.json")
}

pub fn init(root: &Path, wave_n: u32, agents: &[&str]) -> Result<WaveState> {
    std::fs::create_dir_all(root.join(".ctx").join("godmode"))
        .context("failed to create .ctx/godmode directory")?;
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

// ---------------------------------------------------------------------------
// WaveConfig — concurrency, health-check, and retry knobs
// ---------------------------------------------------------------------------

/// Configuration for parallel agent dispatch and health monitoring.
#[derive(Debug, Clone)]
pub struct WaveConfig {
    /// Maximum number of agent slots allowed to run concurrently.
    pub max_concurrency: usize,
    /// Seconds between health-check probes (wall-clock, not enforced here).
    pub health_check_interval_secs: u64,
    /// Maximum retry attempts for a transiently-blocked slot.
    pub max_retries: usize,
    /// Base backoff in milliseconds between retries (doubles each attempt).
    pub retry_backoff_ms: u64,
}

impl Default for WaveConfig {
    fn default() -> Self {
        Self {
            max_concurrency: 5,
            health_check_interval_secs: 30,
            max_retries: 3,
            retry_backoff_ms: 100,
        }
    }
}

// ---------------------------------------------------------------------------
// ConcurrencyTracker — in-process slot counter, independent of WaveState file
// ---------------------------------------------------------------------------

/// Lightweight in-process concurrency gate.
#[derive(Debug)]
pub struct ConcurrencyTracker {
    max: usize,
    active: usize,
}

impl ConcurrencyTracker {
    pub fn new(max: usize) -> Self {
        Self { max, active: 0 }
    }

    /// Returns `true` and increments the active count if a slot is available.
    pub fn try_acquire(&mut self) -> bool {
        if self.active < self.max {
            self.active += 1;
            true
        } else {
            false
        }
    }

    /// Releases a previously acquired slot.
    pub fn release(&mut self) {
        self.active = self.active.saturating_sub(1);
    }

    pub fn active(&self) -> usize {
        self.active
    }

    pub fn available(&self) -> usize {
        self.max.saturating_sub(self.active)
    }
}

// ---------------------------------------------------------------------------
// SlotHealth — per-slot health and retry metadata
// ---------------------------------------------------------------------------

/// Extended per-slot metadata for health monitoring and retry tracking.
#[derive(Debug, Clone)]
pub struct SlotHealth {
    /// Number of transient-block retries attempted so far.
    pub retries: usize,
    /// Whether the last health check passed.
    pub last_check_ok: bool,
}

impl Default for SlotHealth {
    fn default() -> Self {
        Self {
            retries: 0,
            last_check_ok: true,
        }
    }
}

/// Outcome of a `health_check_slot` call.
#[derive(Debug, PartialEq, Eq)]
pub enum HealthOutcome {
    Ok,
    Degraded,
}

/// Check slot health by calling `probe`; returns the outcome and records it in `health`.
pub fn health_check_slot<F>(health: &mut SlotHealth, probe: F) -> HealthOutcome
where
    F: FnOnce() -> bool,
{
    let ok = probe();
    health.last_check_ok = ok;
    if ok {
        HealthOutcome::Ok
    } else {
        HealthOutcome::Degraded
    }
}

/// Outcome of `on_blocked`.
#[derive(Debug, PartialEq, Eq)]
pub enum BlockOutcome {
    /// Retry scheduled (retries < max_retries).
    Retry { attempt: usize },
    /// Max retries exhausted — give up.
    Exhausted,
}

/// Called when a slot reports a transient block.
///
/// Increments `health.retries`; returns `Retry` until `config.max_retries` is reached,
/// then `Exhausted`.
pub fn on_blocked(health: &mut SlotHealth, config: &WaveConfig) -> BlockOutcome {
    health.retries += 1;
    if health.retries <= config.max_retries {
        BlockOutcome::Retry {
            attempt: health.retries,
        }
    } else {
        BlockOutcome::Exhausted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn setup(agents: &[&str]) -> (tempfile::TempDir, WaveState) {
        let dir = tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".ctx").join("godmode")).unwrap();
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

    // --- WaveConfig tests ---

    #[test]
    fn wave_config_defaults() {
        let cfg = WaveConfig::default();
        assert_eq!(cfg.max_concurrency, 5);
        assert_eq!(cfg.health_check_interval_secs, 30);
        assert_eq!(cfg.max_retries, 3);
        assert_eq!(cfg.retry_backoff_ms, 100);
    }

    // --- ConcurrencyTracker tests ---

    #[test]
    fn concurrency_tracker_never_exceeds_max() {
        let mut tracker = ConcurrencyTracker::new(2);
        assert!(tracker.try_acquire());
        assert!(tracker.try_acquire());
        assert!(!tracker.try_acquire(), "must not exceed max=2");
        assert_eq!(tracker.active(), 2);
    }

    #[test]
    fn concurrency_tracker_release_frees_capacity() {
        let mut tracker = ConcurrencyTracker::new(1);
        assert!(tracker.try_acquire());
        assert!(!tracker.try_acquire());
        tracker.release();
        assert!(tracker.try_acquire(), "slot should be free after release");
    }

    #[test]
    fn concurrency_tracker_available_reflects_state() {
        let mut tracker = ConcurrencyTracker::new(3);
        tracker.try_acquire();
        assert_eq!(tracker.available(), 2);
    }

    // --- health_check_slot tests ---

    #[test]
    fn health_check_ok_probe() {
        let mut h = SlotHealth::default();
        let outcome = health_check_slot(&mut h, || true);
        assert_eq!(outcome, HealthOutcome::Ok);
        assert!(h.last_check_ok);
    }

    #[test]
    fn health_check_degraded_probe() {
        let mut h = SlotHealth::default();
        let outcome = health_check_slot(&mut h, || false);
        assert_eq!(outcome, HealthOutcome::Degraded);
        assert!(!h.last_check_ok);
    }

    // --- on_blocked / retry tests ---

    #[test]
    fn on_blocked_retries_up_to_max() {
        let cfg = WaveConfig {
            max_retries: 2,
            ..Default::default()
        };
        let mut h = SlotHealth::default();
        assert_eq!(on_blocked(&mut h, &cfg), BlockOutcome::Retry { attempt: 1 });
        assert_eq!(on_blocked(&mut h, &cfg), BlockOutcome::Retry { attempt: 2 });
        assert_eq!(on_blocked(&mut h, &cfg), BlockOutcome::Exhausted);
    }

    #[test]
    fn on_blocked_exhausted_after_max_retries() {
        let cfg = WaveConfig {
            max_retries: 1,
            ..Default::default()
        };
        let mut h = SlotHealth::default();
        on_blocked(&mut h, &cfg); // attempt 1 — Retry
        let outcome = on_blocked(&mut h, &cfg); // attempt 2 — Exhausted
        assert_eq!(outcome, BlockOutcome::Exhausted);
    }
}
