//! Property-based tests for `wave` slot state machine and concurrency tracker.

#[cfg(test)]
mod wave_properties {
    use godmode_core::wave::{
        self, BlockOutcome, ConcurrencyTracker, SlotHealth, SlotStatus, WaveConfig, WaveState,
    };
    use proptest::prelude::*;
    use std::collections::BTreeMap;

    // -- WaveState property strategies --

    fn arb_slot_status() -> impl Strategy<Value = SlotStatus> {
        prop_oneof![
            Just(SlotStatus::Pending),
            Just(SlotStatus::Done),
            Just(SlotStatus::Blocked),
        ]
    }

    fn arb_wave_state(max_agents: usize) -> impl Strategy<Value = WaveState> {
        (
            1u32..=10,
            prop::collection::vec(arb_slot_status(), 1..=max_agents),
        )
            .prop_map(|(wave_n, statuses)| {
                let mut agents = BTreeMap::new();
                for (i, status) in statuses.into_iter().enumerate() {
                    let name = format!("agent-{}", i + 1);
                    agents.insert(
                        name.clone(),
                        wave::AgentSlot {
                            status,
                            branch: name,
                            commits: vec![],
                        },
                    );
                }
                WaveState {
                    wave: wave_n,
                    agents,
                }
            })
    }

    proptest! {
        /// check() returns true iff no slot is Pending.
        #[test]
        fn check_consistent_with_slots(state in arb_wave_state(8)) {
            let has_pending = state
                .agents
                .values()
                .any(|s| s.status == SlotStatus::Pending);
            prop_assert_eq!(wave::check(&state), !has_pending);
        }
    }

    proptest! {
        /// all_done() returns true iff every slot is Done.
        #[test]
        fn all_done_consistent_with_slots(state in arb_wave_state(8)) {
            let every_done = state
                .agents
                .values()
                .all(|s| s.status == SlotStatus::Done);
            prop_assert_eq!(wave::all_done(&state), every_done);
        }
    }

    proptest! {
        /// all_done implies check (settled superset of all-done).
        #[test]
        fn all_done_implies_check(state in arb_wave_state(8)) {
            if wave::all_done(&state) {
                prop_assert!(wave::check(&state), "all_done should imply check");
            }
        }
    }

    proptest! {
        /// init creates exactly N agents, all Pending.
        #[test]
        fn init_all_pending(n in 1usize..=8) {
            let dir = tempfile::TempDir::new().unwrap();
            let names: Vec<String> = (0..n).map(|i| format!("a{i}")).collect();
            let name_refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
            let state = wave::init(dir.path(), 1, &name_refs).unwrap();
            prop_assert_eq!(state.agents.len(), n);
            for slot in state.agents.values() {
                prop_assert_eq!(&slot.status, &SlotStatus::Pending);
            }
        }
    }

    // -- ConcurrencyTracker properties --

    /// Actions on ConcurrencyTracker.
    #[derive(Debug, Clone)]
    enum TrackerAction {
        Acquire,
        Release,
    }

    fn arb_tracker_actions() -> impl Strategy<Value = Vec<TrackerAction>> {
        prop::collection::vec(
            prop_oneof![Just(TrackerAction::Acquire), Just(TrackerAction::Release)],
            0..30,
        )
    }

    proptest! {
        /// active() never exceeds max, regardless of action sequence.
        #[test]
        fn tracker_never_exceeds_max(
            max in 1usize..=10,
            actions in arb_tracker_actions(),
        ) {
            let mut tracker = ConcurrencyTracker::new(max);
            for action in &actions {
                match action {
                    TrackerAction::Acquire => { tracker.try_acquire(); }
                    TrackerAction::Release => { tracker.release(); }
                }
                prop_assert!(
                    tracker.active() <= max,
                    "active {} exceeds max {}",
                    tracker.active(),
                    max
                );
            }
        }
    }

    proptest! {
        /// active() + available() always equals max.
        #[test]
        fn tracker_active_plus_available_equals_max(
            max in 1usize..=10,
            actions in arb_tracker_actions(),
        ) {
            let mut tracker = ConcurrencyTracker::new(max);
            for action in &actions {
                match action {
                    TrackerAction::Acquire => { tracker.try_acquire(); }
                    TrackerAction::Release => { tracker.release(); }
                }
                prop_assert_eq!(
                    tracker.active() + tracker.available(),
                    max,
                    "active + available must equal max"
                );
            }
        }
    }

    proptest! {
        /// release() never underflows (saturating_sub).
        #[test]
        fn tracker_release_never_underflows(releases in 1usize..20) {
            let mut tracker = ConcurrencyTracker::new(1);
            for _ in 0..releases {
                tracker.release();
            }
            prop_assert_eq!(tracker.active(), 0);
        }
    }

    // -- on_blocked retry properties --

    proptest! {
        /// on_blocked returns Retry exactly max_retries times, then Exhausted.
        #[test]
        fn on_blocked_retry_count_matches_config(max_retries in 0usize..=10) {
            let cfg = WaveConfig {
                max_retries,
                ..Default::default()
            };
            let mut health = SlotHealth::default();
            let mut retry_count = 0usize;
            while let BlockOutcome::Retry { .. } = wave::on_blocked(&mut health, &cfg) {
                retry_count += 1;
            }
            prop_assert_eq!(retry_count, max_retries);
        }
    }
}
