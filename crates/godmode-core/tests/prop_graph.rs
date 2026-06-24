//! Property-based tests for `graph`, `plan`, and `dispatch`.
#![allow(clippy::needless_range_loop)]
//!
//! Run with: cargo nextest run --test prop_graph

use godmode_core::model::{Status, Task, TaskGraph};
use godmode_core::{dispatch, graph, plan};
use proptest::prelude::*;

// ── Strategies ───────────────────────────────────────────────────────────────

/// Generate a valid task ID: lowercase alphanumeric, 1-8 chars.
fn task_id() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9]{0,7}".prop_map(|s| s)
}

/// Generate a non-empty title string (printable ASCII, no newlines).
fn task_title() -> impl Strategy<Value = String> {
    "[A-Za-z][A-Za-z0-9 ]{1,30}".prop_map(|s| s)
}

/// Generate a Status value.
fn arb_status() -> impl Strategy<Value = Status> {
    prop_oneof![
        Just(Status::Pending),
        Just(Status::Running),
        Just(Status::Done),
        Just(Status::Blocked),
    ]
}

/// Build a TaskGraph with 0-8 tasks, each with a unique ID and arbitrary status.
/// No dependency edges — suitable for tests that don't need DAG structure.
fn arb_flat_graph() -> impl Strategy<Value = TaskGraph> {
    prop::collection::vec((task_id(), task_title(), arb_status()), 0..=8).prop_filter_map(
        "unique IDs",
        |specs| {
            let mut g = TaskGraph::default();
            let mut seen = std::collections::HashSet::new();
            for (id, title, status) in specs {
                if !seen.insert(id.clone()) {
                    return None; // duplicate ID — discard sample
                }
                let mut t = Task::new(id, title);
                t.status = status;
                g.tasks.push(t);
            }
            Some(g)
        },
    )
}

/// Build a strictly sequential chain of N pending tasks: t1 → t2 → … → tN.
#[allow(dead_code)]
fn arb_chain(max_len: usize) -> impl Strategy<Value = TaskGraph> {
    (1..=max_len).prop_map(|n| {
        let mut g = TaskGraph::default();
        for i in 1..=n {
            let mut t = Task::new(format!("t{i}"), format!("Task {i}"));
            if i > 1 {
                t.depends_on = vec![format!("t{}", i - 1)];
            }
            g.tasks.push(t);
        }
        g
    })
}

/// Build a DAG by adding tasks sequentially, each depending on a random subset
/// of the tasks already in the graph. Always acyclic by construction.
fn arb_dag(max_tasks: usize) -> impl Strategy<Value = TaskGraph> {
    (1..=max_tasks)
        .prop_flat_map(|n| {
            // For each task i (1-indexed), generate a bitmask selecting deps from 1..i-1.
            prop::collection::vec(any::<u32>(), n).prop_map(move |masks| (n, masks))
        })
        .prop_map(|(n, masks)| {
            let mut g = TaskGraph::default();
            for i in 0..n {
                let id = format!("t{}", i + 1);
                let title = format!("Task {}", i + 1);
                let mut t = Task::new(id, title);
                // Pick deps from tasks already added (indices 0..i).
                if i > 0 {
                    let mask = masks[i];
                    for j in 0..i {
                        if (mask >> j) & 1 == 1 {
                            t.depends_on.push(format!("t{}", j + 1));
                        }
                    }
                }
                g.tasks.push(t);
            }
            g
        })
}

// ── graph: summary invariants ─────────────────────────────────────────────────

proptest! {
    /// summary counts always sum to tasks.len()
    #[test]
    fn prop_summary_counts_sum_to_len(g in arb_flat_graph()) {
        let s = g.summary();
        prop_assert_eq!(
            s.done + s.running + s.pending + s.blocked,
            g.tasks.len()
        );
    }
}

proptest! {
    /// A graph with only Pending tasks: summary.pending == tasks.len(), others 0.
    #[test]
    fn prop_summary_all_pending(titles in prop::collection::vec(task_title(), 0..=8)) {
        let mut g = TaskGraph::default();
        for (i, title) in titles.iter().enumerate() {
            let mut t = Task::new(format!("t{i}"), title.clone());
            t.status = Status::Pending;
            g.tasks.push(t);
        }
        let s = g.summary();
        prop_assert_eq!(s.pending, g.tasks.len());
        prop_assert_eq!(s.done, 0);
        prop_assert_eq!(s.running, 0);
        prop_assert_eq!(s.blocked, 0);
    }
}

// ── graph: add/remove roundtrip ───────────────────────────────────────────────

proptest! {
    /// add then remove returns graph to original size.
    #[test]
    fn prop_add_remove_roundtrip(
        mut g in arb_flat_graph(),
        id in "[a-z][a-z0-9]{0,7}",
        title in task_title(),
    ) {
        // Ensure ID doesn't clash with existing tasks.
        prop_assume!(!g.tasks.iter().any(|t| t.id == id));
        let before = g.tasks.len();
        graph::add(&mut g, Task::new(id.clone(), title)).unwrap();
        prop_assert_eq!(g.tasks.len(), before + 1);
        graph::remove(&mut g, &id).unwrap();
        prop_assert_eq!(g.tasks.len(), before);
    }
}

proptest! {
    /// add rejects duplicate IDs for any arbitrary ID.
    #[test]
    fn prop_add_duplicate_always_fails(id in task_id(), title in task_title()) {
        let mut g = TaskGraph::default();
        graph::add(&mut g, Task::new(id.clone(), title.clone())).unwrap();
        let err = graph::add(&mut g, Task::new(id.clone(), title)).unwrap_err();
        prop_assert!(err.to_string().contains("already exists"));
    }
}

// ── graph: runnable invariants ────────────────────────────────────────────────

proptest! {
    /// runnable() never returns tasks that aren't Pending.
    #[test]
    fn prop_runnable_only_returns_pending(g in arb_flat_graph()) {
        for task in graph::runnable(&g) {
            prop_assert_eq!(&task.status, &Status::Pending);
        }
    }
}

proptest! {
    /// runnable() never returns a task with an unmet (non-Done) dependency.
    #[test]
    fn prop_runnable_deps_all_done(g in arb_dag(8)) {
        let done_ids: std::collections::HashSet<&str> = g
            .tasks
            .iter()
            .filter(|t| t.status == Status::Done)
            .map(|t| t.id.as_str())
            .collect();
        for task in graph::runnable(&g) {
            for dep in &task.depends_on {
                prop_assert!(
                    done_ids.contains(dep.as_str()),
                    "runnable task '{}' has unmet dep '{}'",
                    task.id,
                    dep
                );
            }
        }
    }
}

// ── graph: unblock_all invariants ─────────────────────────────────────────────

proptest! {
    /// unblock_all only changes Blocked→Pending; Done/Running/Pending tasks untouched.
    #[test]
    fn prop_unblock_all_only_touches_blocked(mut g in arb_flat_graph()) {
        let before: Vec<(String, Status)> = g
            .tasks
            .iter()
            .map(|t| (t.id.clone(), t.status.clone()))
            .collect();

        let count = graph::unblock_all(&mut g);
        let blocked_before = before.iter().filter(|(_, s)| *s == Status::Blocked).count();
        prop_assert_eq!(count, blocked_before);

        for (before_id, before_status) in &before {
            let after = g.tasks.iter().find(|t| t.id == *before_id).unwrap();
            if *before_status == Status::Blocked {
                prop_assert_eq!(&after.status, &Status::Pending);
            } else {
                prop_assert_eq!(&after.status, before_status);
            }
        }
    }
}

// ── graph: clear invariants ───────────────────────────────────────────────────

proptest! {
    /// clear(done_only=true) never removes non-Done tasks.
    #[test]
    fn prop_clear_done_only_preserves_non_done(mut g in arb_flat_graph()) {
        let non_done_before: Vec<String> = g
            .tasks
            .iter()
            .filter(|t| t.status != Status::Done)
            .map(|t| t.id.clone())
            .collect();

        graph::clear(&mut g, true);

        let remaining_ids: std::collections::HashSet<&str> =
            g.tasks.iter().map(|t| t.id.as_str()).collect();

        for id in &non_done_before {
            prop_assert!(
                remaining_ids.contains(id.as_str()),
                "clear(done_only) removed non-Done task '{id}'"
            );
        }
    }
}

proptest! {
    /// clear(done_only=false) always empties the graph.
    #[test]
    fn prop_clear_all_empties_graph(mut g in arb_flat_graph()) {
        graph::clear(&mut g, false);
        prop_assert!(g.tasks.is_empty());
    }
}

proptest! {
    /// clear returns the correct removed count.
    #[test]
    fn prop_clear_count_correct(mut g in arb_flat_graph()) {
        let done_count = g.tasks.iter().filter(|t| t.status == Status::Done).count();
        let removed = graph::clear(&mut g, true);
        prop_assert_eq!(removed, done_count);
    }
}

// ── graph: acyclic DAG construction ──────────────────────────────────────────

proptest! {
    /// Adding tasks from a pre-built DAG (sequential deps) never produces a cycle error.
    #[test]
    fn prop_sequential_dag_never_cycles(n in 1usize..=10) {
        let mut g = TaskGraph::default();
        for i in 1..=n {
            let mut t = Task::new(format!("t{i}"), format!("Task {i}"));
            if i > 1 {
                t.depends_on = vec![format!("t{}", i - 1)];
            }
            graph::add(&mut g, t).expect("sequential DAG should never cycle");
        }
        prop_assert_eq!(g.tasks.len(), n);
    }
}

// ── plan::parse invariants ────────────────────────────────────────────────────

/// Build a plan markdown string with N `### Task N: <title>` headings.
fn make_plan(titles: &[String]) -> String {
    let mut s = String::from("# Plan\n\n");
    for (i, title) in titles.iter().enumerate() {
        s.push_str(&format!("### Task {}: {}\n\n", i + 1, title));
    }
    s
}

proptest! {
    /// N headings produce exactly N tasks.
    #[test]
    fn prop_parse_task_count(titles in prop::collection::vec(task_title(), 0..=10)) {
        let md = make_plan(&titles);
        let tasks = plan::parse(&md).unwrap();
        prop_assert_eq!(tasks.len(), titles.len());
    }
}

proptest! {
    /// Tasks have sequential IDs t1..tN.
    #[test]
    fn prop_parse_sequential_ids(titles in prop::collection::vec(task_title(), 1..=10)) {
        let md = make_plan(&titles);
        let tasks = plan::parse(&md).unwrap();
        for (i, task) in tasks.iter().enumerate() {
            prop_assert_eq!(&task.id, &format!("t{}", i + 1));
        }
    }
}

proptest! {
    /// First task has no deps; each subsequent task depends on the previous one.
    #[test]
    fn prop_parse_sequential_deps(titles in prop::collection::vec(task_title(), 2..=10)) {
        let md = make_plan(&titles);
        let tasks = plan::parse(&md).unwrap();
        prop_assert!(tasks[0].depends_on.is_empty(), "first task should have no deps");
        for i in 1..tasks.len() {
            prop_assert_eq!(
                &tasks[i].depends_on,
                &vec![format!("t{}", i)],
                "task t{} should depend on t{}",
                i + 1,
                i
            );
        }
    }
}

// ── dispatch invariants ───────────────────────────────────────────────────────

proptest! {
    /// independent_chains never returns more chains than max_concurrent.
    #[test]
    fn prop_chains_respect_max_concurrent(
        g in arb_dag(12),
        max in 1usize..=5,
    ) {
        let chains = dispatch::independent_chains(&g, max);
        prop_assert!(
            chains.len() <= max,
            "got {} chains, expected <= {}",
            chains.len(),
            max
        );
    }
}

proptest! {
    /// Every task ID in any chain exists in the original graph.
    #[test]
    fn prop_chains_only_contain_known_tasks(g in arb_dag(10)) {
        let all_ids: std::collections::HashSet<&str> =
            g.tasks.iter().map(|t| t.id.as_str()).collect();
        let chains = dispatch::independent_chains(&g, 5);
        for chain in &chains {
            for task_ref in &chain.tasks {
                prop_assert!(
                    all_ids.contains(task_ref.id.as_str()),
                    "chain contains unknown task id '{}'",
                    task_ref.id
                );
            }
        }
    }
}

proptest! {
    /// No task ID appears in two different chains.
    #[test]
    fn prop_chains_are_disjoint(g in arb_dag(10)) {
        let chains = dispatch::independent_chains(&g, 5);
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        for chain in &chains {
            for task_ref in &chain.tasks {
                prop_assert!(
                    seen.insert(task_ref.id.clone()),
                    "task '{}' appears in multiple chains",
                    task_ref.id
                );
            }
        }
    }
}

proptest! {
    /// critical_path length is always <= total pending/running task count.
    #[test]
    fn prop_critical_path_length_bounded(g in arb_dag(10)) {
        let active_count = g
            .tasks
            .iter()
            .filter(|t| matches!(t.status, Status::Pending | Status::Running))
            .count();
        let path = dispatch::critical_path(&g);
        prop_assert!(
            path.len() <= active_count,
            "critical path ({}) longer than active task count ({})",
            path.len(),
            active_count
        );
    }
}

proptest! {
    /// critical_path tasks all exist in the original graph.
    #[test]
    fn prop_critical_path_tasks_exist(g in arb_dag(10)) {
        let all_ids: std::collections::HashSet<&str> =
            g.tasks.iter().map(|t| t.id.as_str()).collect();
        for task_ref in dispatch::critical_path(&g) {
            prop_assert!(
                all_ids.contains(task_ref.id.as_str()),
                "critical path contains unknown task '{}'",
                task_ref.id
            );
        }
    }
}
