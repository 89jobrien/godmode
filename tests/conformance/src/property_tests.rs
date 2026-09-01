//! Property-based conformance tests for task graphs and plan parsing.
//!
//! The graph properties cover generated task identifiers, reversible graph
//! mutations, cycle rejection, and dependency-aware task selection. The plan
//! properties cover task extraction and the sequential dependency chain created
//! from numbered plan headings.
//!
//! Proptest exposes these cases as standard `#[test]` functions.

#[cfg(test)]
mod graph_properties {
    use godmode_core::graph;
    use godmode_core::model::{Task, TaskGraph};
    use proptest::prelude::*;

    fn arb_task_id() -> impl Strategy<Value = String> {
        "[a-z]{1,4}[0-9]{1,2}".prop_map(|s| s)
    }

    fn arb_title() -> impl Strategy<Value = String> {
        "[A-Za-z ]{3,20}".prop_map(|s| s)
    }

    proptest! {
        /// next_task_id always returns an ID not already in the graph.
        #[test]
        fn next_id_never_collides(existing in prop::collection::vec(arb_task_id(), 0..10)) {
            let mut g = TaskGraph::default();
            for id in &existing {
                // add tasks with unique IDs (skip duplicates)
                if !g.tasks.iter().any(|t| &t.id == id) {
                    g.tasks.push(Task::new(id.clone(), "x"));
                }
            }
            let next = graph::next_task_id(&g).unwrap();
            prop_assert!(!g.tasks.iter().any(|t| t.id == next));
        }

        /// Adding a task and then removing it leaves the graph unchanged.
        #[test]
        fn add_then_remove_is_identity(id in arb_task_id(), title in arb_title()) {
            let mut g = TaskGraph::default();
            let before_len = g.tasks.len();
            graph::add(&mut g, Task::new(id.clone(), title)).unwrap();
            prop_assert_eq!(g.tasks.len(), before_len + 1);
            graph::remove(&mut g, &id).unwrap();
            prop_assert_eq!(g.tasks.len(), before_len);
        }

        /// A self-loop always produces a cycle error.
        #[test]
        fn self_loop_always_cycle(id in arb_task_id()) {
            let mut g = TaskGraph::default();
            let mut t = Task::new(id.clone(), "x");
            t.depends_on = vec![id.clone()];
            let result = graph::add(&mut g, t);
            prop_assert!(result.is_err());
            prop_assert!(result.unwrap_err().to_string().contains("cycle"));
        }

        /// runnable() never returns a task whose dependencies are not done.
        #[test]
        fn runnable_always_has_satisfied_deps(
            ids in prop::collection::vec("[a-z]{2}", 1..6)
        ) {
            let unique_ids: Vec<String> = {
                let mut seen = std::collections::HashSet::new();
                ids.into_iter().filter(|id| seen.insert(id.clone())).collect()
            };
            if unique_ids.is_empty() { return Ok(()); }
            let mut g = TaskGraph::default();
            for id in &unique_ids {
                g.tasks.push(Task::new(id.clone(), "x"));
            }
            let runnable = graph::runnable(&g);
            let done_ids: std::collections::HashSet<&str> = g.tasks.iter()
                .filter(|t| t.status == godmode_core::model::Status::Done)
                .map(|t| t.id.as_str())
                .collect();
            for t in runnable {
                for dep in &t.depends_on {
                    prop_assert!(done_ids.contains(dep.as_str()),
                        "runnable task {} has unsatisfied dep {}", t.id, dep);
                }
            }
        }
    }
}

#[cfg(test)]
mod plan_properties {
    use godmode_core::plan;
    use proptest::prelude::*;

    fn arb_task_title() -> impl Strategy<Value = String> {
        "[A-Za-z ]{3,15}".prop_map(|s| s)
    }

    proptest! {
        /// Plan parse: N headings always produce N tasks.
        #[test]
        fn n_headings_produce_n_tasks(
            titles in prop::collection::vec(arb_task_title(), 1..8)
        ) {
            let md: String = titles.iter().enumerate()
                .map(|(i, t)| format!("### Task {}: {}\n", i + 1, t))
                .collect();
            let tasks = plan::parse(&md).unwrap();
            prop_assert_eq!(tasks.len(), titles.len());
        }

        /// Plan parse: sequential deps form a chain (task N depends on task N-1).
        #[test]
        fn tasks_form_sequential_chain(
            n in 2usize..6
        ) {
            let md: String = (1..=n)
                .map(|i| format!("### Task {}: Task{}\n", i, i))
                .collect();
            let tasks = plan::parse(&md).unwrap();
            prop_assert_eq!(tasks[0].depends_on.len(), 0);
            for i in 1..n {
                prop_assert_eq!(tasks[i].depends_on.len(), 1);
                prop_assert_eq!(&tasks[i].depends_on[0], &tasks[i-1].id);
            }
        }
    }
}
