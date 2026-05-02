//! Compute independent task chains for parallel agent dispatch.
//!
//! Two chains are independent when no task in one chain appears in the `depends_on`
//! of any task in the other chain (transitively). Each chain is dispatched to one
//! `tdd-crate-agent`. Maximum 5 concurrent chains.

use std::collections::{HashMap, HashSet};

use crate::model::{Status, Task, TaskGraph};

/// A task reference with id and title for orca-strait agent consumption.
#[derive(Debug, Clone, serde::Serialize)]
pub struct TaskRef {
    pub id: String,
    pub title: String,
}

/// A group of tasks that must execute sequentially, targeting one crate.
#[derive(Debug, serde::Serialize)]
pub struct Chain {
    /// Crate this chain targets (if all tasks share a crate name).
    pub crate_name: Option<String>,
    /// Tasks in execution order with id + title.
    pub tasks: Vec<TaskRef>,
}

/// Decompose the graph into independent chains of runnable + pending tasks.
///
/// Returns at most `max_concurrent` chains (default 5).
pub fn independent_chains(graph: &TaskGraph, max_concurrent: usize) -> Vec<Chain> {
    // Only consider pending/running tasks — done/blocked are settled.
    let active: Vec<&Task> = graph
        .tasks
        .iter()
        .filter(|t| matches!(t.status, Status::Pending | Status::Running))
        .collect();

    if active.is_empty() {
        return vec![];
    }

    // Build adjacency: id -> deps (within active set only).
    let active_ids: HashSet<&str> = active.iter().map(|t| t.id.as_str()).collect();
    let deps_map: HashMap<&str, Vec<&str>> = active
        .iter()
        .map(|t| {
            let deps: Vec<&str> = t
                .depends_on
                .iter()
                .filter(|d| active_ids.contains(d.as_str()))
                .map(|d| d.as_str())
                .collect();
            (t.id.as_str(), deps)
        })
        .collect();

    // Identify root tasks (no active dependencies).
    let roots: Vec<&str> = active
        .iter()
        .filter(|t| deps_map[t.id.as_str()].is_empty())
        .map(|t| t.id.as_str())
        .collect();

    // Build chains by following each root forward through its dependents.
    let mut chains: Vec<Chain> = Vec::new();
    let mut claimed: HashSet<&str> = HashSet::new();

    // Build reverse map: id -> tasks that depend on it.
    let mut rev: HashMap<&str, Vec<&str>> = HashMap::new();
    for (id, deps) in &deps_map {
        for dep in deps {
            rev.entry(dep).or_default().push(id);
        }
    }

    for root in roots {
        if claimed.contains(root) {
            continue;
        }
        let mut chain_ids = vec![root];
        claimed.insert(root);

        // Follow single-successor chains.
        loop {
            let last = *chain_ids.last().unwrap();
            let successors: Vec<&str> = rev
                .get(last)
                .map(|v| v.iter().copied().filter(|s| !claimed.contains(s)).collect())
                .unwrap_or_default();
            if successors.len() == 1 {
                let next = successors[0];
                chain_ids.push(next);
                claimed.insert(next);
            } else {
                break;
            }
        }

        let crate_name = infer_crate(&chain_ids, graph);
        let tasks = chain_ids
            .iter()
            .filter_map(|id| graph.tasks.iter().find(|t| t.id == *id))
            .map(|t| TaskRef {
                id: t.id.clone(),
                title: t.title.clone(),
            })
            .collect();
        chains.push(Chain { crate_name, tasks });

        if chains.len() >= max_concurrent {
            break;
        }
    }

    chains
}

fn infer_crate(ids: &[&str], graph: &TaskGraph) -> Option<String> {
    let crates: HashSet<Option<&str>> = ids
        .iter()
        .filter_map(|id| graph.tasks.iter().find(|t| t.id == *id))
        .map(|t| t.crate_name.as_deref())
        .collect();
    // If all tasks share exactly one crate name, return it.
    if crates.len() == 1 {
        crates.into_iter().next().flatten().map(str::to_string)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Task;

    fn make_graph(specs: &[(&str, &str, &[&str], Option<&str>)]) -> TaskGraph {
        let mut g = TaskGraph::default();
        for (id, title, deps, krate) in specs {
            let mut t = Task::new(*id, *title);
            t.depends_on = deps.iter().map(|s| s.to_string()).collect();
            t.crate_name = krate.map(str::to_string);
            g.tasks.push(t);
        }
        g
    }

    #[test]
    fn single_chain() {
        let g = make_graph(&[
            ("t1", "A", &[], Some("foo")),
            ("t2", "B", &["t1"], Some("foo")),
        ]);
        let chains = independent_chains(&g, 5);
        assert_eq!(chains.len(), 1);
        let ids: Vec<&str> = chains[0].tasks.iter().map(|t| t.id.as_str()).collect();
        assert_eq!(ids, vec!["t1", "t2"]);
        assert_eq!(chains[0].crate_name.as_deref(), Some("foo"));
    }

    #[test]
    fn two_independent_chains() {
        let g = make_graph(&[
            ("a1", "A1", &[], Some("auth")),
            ("a2", "A2", &["a1"], Some("auth")),
            ("b1", "B1", &[], Some("cache")),
            ("b2", "B2", &["b1"], Some("cache")),
        ]);
        let chains = independent_chains(&g, 5);
        assert_eq!(chains.len(), 2);
    }

    #[test]
    fn respects_max_concurrent() {
        let g = make_graph(&[
            ("t1", "A", &[], None),
            ("t2", "B", &[], None),
            ("t3", "C", &[], None),
            ("t4", "D", &[], None),
            ("t5", "E", &[], None),
            ("t6", "F", &[], None),
        ]);
        let chains = independent_chains(&g, 5);
        assert!(chains.len() <= 5);
    }

    #[test]
    fn empty_graph_returns_no_chains() {
        let g = TaskGraph::default();
        let chains = independent_chains(&g, 5);
        assert!(chains.is_empty());
    }
}
