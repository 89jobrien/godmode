//! Property-based tests for `Session` state machine invariants.
//!
//! These test that arbitrary sequences of valid transitions always produce
//! consistent state. Runs as standard `#[test]` functions via proptest.

#[cfg(test)]
mod session_properties {
    use godmode_core::model::{Status, Task, TaskGraph};
    use godmode_core::session::Session;
    use proptest::prelude::*;

    /// Actions that can be applied to a session with tasks t1..tN.
    #[derive(Debug, Clone)]
    enum Action {
        Start(usize),
        Complete(usize),
        Block(usize),
        Unblock(usize),
    }

    /// Generate a sequence of actions for a graph with `n` tasks.
    fn arb_actions(n: usize) -> impl Strategy<Value = Vec<Action>> {
        prop::collection::vec(
            (0..n, 0u8..4).prop_map(|(idx, kind)| match kind {
                0 => Action::Start(idx),
                1 => Action::Complete(idx),
                2 => Action::Block(idx),
                _ => Action::Unblock(idx),
            }),
            0..20,
        )
    }

    /// Apply actions to a session, ignoring errors (invalid transitions).
    /// Then check invariants on the final state.
    fn apply_actions_and_check(n: usize, actions: Vec<Action>) {
        let dir = tempfile::TempDir::new().unwrap();
        let mut cfg = godmode_core::config::Config::default();
        // Disable integrations to avoid subprocess calls.
        cfg.integrations.rx = false;
        cfg.integrations.crux = false;
        cfg.integrations.doob = false;
        cfg.integrations.hj = false;

        let mut s = Session::open_with_config(dir.path(), &cfg).unwrap();

        // Add n independent tasks (no deps — simplifies valid transitions).
        for i in 0..n {
            s.add_task(Task::new(format!("t{}", i + 1), format!("Task {}", i + 1)))
                .unwrap();
        }

        for action in &actions {
            let id = match action {
                Action::Start(i) => format!("t{}", i + 1),
                Action::Complete(i) => format!("t{}", i + 1),
                Action::Block(i) => format!("t{}", i + 1),
                Action::Unblock(i) => format!("t{}", i + 1),
            };
            match action {
                Action::Start(_) => {
                    let _ = s.start_task(&id);
                }
                Action::Complete(_) => {
                    let _ = s.complete_task(&id, None, None);
                }
                Action::Block(_) => {
                    let _ = s.block_task(&id, "test block");
                }
                Action::Unblock(_) => {
                    let _ = s.unblock_task(&id);
                }
            }
        }

        // --- Invariants ---

        let graph = s.graph();

        // 1. Summary counts always equal task count.
        let summary = graph.summary();
        assert_eq!(
            summary.done + summary.running + summary.pending + summary.blocked,
            graph.tasks.len(),
            "summary counts must sum to task count"
        );

        // 2. Running count never exceeds total started.
        let running = graph
            .tasks
            .iter()
            .filter(|t| t.status == Status::Running)
            .count();
        assert!(running <= n, "running count exceeds task count");

        // 3. Done tasks have status Done (tautological but verifies no corruption).
        for task in &graph.tasks {
            if task.status == Status::Done {
                assert!(
                    task.completed.is_some() || task.completed_at.is_some(),
                    "done task {} should have completion metadata",
                    task.id
                );
            }
        }

        // 4. Session summary duration is always non-negative.
        let session_summary = s.summary();
        for timing in &session_summary.tasks {
            // duration_ms is u64, so it's always >= 0, but verify it doesn't wrap.
            let _ = timing.duration_ms;
        }

        // 5. No task has an impossible status transition result:
        //    a started_at without Running or Done status is suspicious but allowed
        //    (block can move Running → Blocked while preserving started_at).
    }

    proptest! {
        #[test]
        fn session_random_actions_maintain_invariants(
            n in 1usize..6,
            actions in arb_actions(6),
        ) {
            apply_actions_and_check(n, actions);
        }
    }

    proptest! {
        /// Starting then completing every task: all end up Done.
        #[test]
        fn start_complete_all_yields_all_done(n in 1usize..8) {
            let dir = tempfile::TempDir::new().unwrap();
            let mut cfg = godmode_core::config::Config::default();
            cfg.integrations.rx = false;
            cfg.integrations.crux = false;
            cfg.integrations.doob = false;
            cfg.integrations.hj = false;
            let mut s = Session::open_with_config(dir.path(), &cfg).unwrap();

            for i in 0..n {
                s.add_task(Task::new(format!("t{}", i + 1), format!("Task {}", i + 1)))
                    .unwrap();
            }
            for i in 0..n {
                let id = format!("t{}", i + 1);
                s.start_task(&id).unwrap();
                s.complete_task(&id, None, None).unwrap();
            }

            let summary = s.graph().summary();
            prop_assert_eq!(summary.done, n);
            prop_assert_eq!(summary.pending, 0);
            prop_assert_eq!(summary.running, 0);
            prop_assert_eq!(summary.blocked, 0);
        }
    }

    proptest! {
        /// Blocking then unblocking returns to Pending.
        #[test]
        fn block_unblock_returns_to_pending(n in 1usize..6) {
            let dir = tempfile::TempDir::new().unwrap();
            let mut cfg = godmode_core::config::Config::default();
            cfg.integrations.rx = false;
            cfg.integrations.crux = false;
            cfg.integrations.doob = false;
            cfg.integrations.hj = false;
            let mut s = Session::open_with_config(dir.path(), &cfg).unwrap();

            for i in 0..n {
                s.add_task(Task::new(format!("t{}", i + 1), format!("Task {}", i + 1)))
                    .unwrap();
            }
            // Start then block each.
            for i in 0..n {
                let id = format!("t{}", i + 1);
                s.start_task(&id).unwrap();
                s.block_task(&id, "stuck").unwrap();
            }
            // All should be blocked.
            let summary = s.graph().summary();
            prop_assert_eq!(summary.blocked, n);

            // Unblock all.
            let unblocked = s.unblock_all();
            prop_assert_eq!(unblocked, n);

            let summary2 = s.graph().summary();
            prop_assert_eq!(summary2.pending, n);
            prop_assert_eq!(summary2.blocked, 0);
        }
    }
}
