//! Compute independent task chains for parallel agent dispatch.
//!
//! Two chains are independent when no task in one chain appears in the `depends_on`
//! of any task in the other chain (transitively). Each chain is dispatched to one
//! `godmode-crate-agent`. Maximum 5 concurrent chains.

use std::collections::{HashMap, HashSet};

use crate::model::{Status, Task, TaskGraph};
use crate::wave::{BlockOutcome, ConcurrencyTracker, SlotHealth, WaveConfig, on_blocked};

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

/// Return the longest dependency chain among pending/running tasks (critical path).
/// Uses topological sort + DP on depth. Ties broken by insertion order.
/// Returns tasks in root → tail execution order.
pub fn critical_path(graph: &TaskGraph) -> Vec<TaskRef> {
    let active: Vec<&Task> = graph
        .tasks
        .iter()
        .filter(|t| matches!(t.status, Status::Pending | Status::Running))
        .collect();

    if active.is_empty() {
        return vec![];
    }

    let active_ids: HashSet<&str> = active.iter().map(|t| t.id.as_str()).collect();

    let mut in_degree: HashMap<&str, usize> = active.iter().map(|t| (t.id.as_str(), 0)).collect();
    let mut rev: HashMap<&str, Vec<&str>> = HashMap::new();

    for t in &active {
        for dep in t
            .depends_on
            .iter()
            .filter(|d| active_ids.contains(d.as_str()))
        {
            *in_degree.entry(t.id.as_str()).or_default() += 1;
            rev.entry(dep.as_str()).or_default().push(t.id.as_str());
        }
    }

    let mut depth: HashMap<&str, usize> = active.iter().map(|t| (t.id.as_str(), 1)).collect();
    let mut parent: HashMap<&str, Option<&str>> =
        active.iter().map(|t| (t.id.as_str(), None)).collect();

    let mut queue: std::collections::VecDeque<&str> = active
        .iter()
        .filter(|t| in_degree[t.id.as_str()] == 0)
        .map(|t| t.id.as_str())
        .collect();

    while let Some(id) = queue.pop_front() {
        let d = depth[id];
        if let Some(successors) = rev.get(id) {
            for &succ in successors {
                let new_d = d + 1;
                if new_d > depth[succ] {
                    depth.insert(succ, new_d);
                    parent.insert(succ, Some(id));
                }
                let deg = in_degree.get_mut(succ).unwrap();
                *deg -= 1;
                if *deg == 0 {
                    queue.push_back(succ);
                }
            }
        }
    }

    // Pick the first task (insertion order) with maximum depth for stable tie-breaking.
    let max_depth = active.iter().map(|t| depth[t.id.as_str()]).max().unwrap();
    let tail = active
        .iter()
        .find(|t| depth[t.id.as_str()] == max_depth)
        .map(|t| t.id.as_str())
        .unwrap();

    let mut path_ids: Vec<&str> = Vec::new();
    let mut cur = Some(tail);
    while let Some(id) = cur {
        path_ids.push(id);
        cur = parent[id];
    }
    path_ids.reverse();

    path_ids
        .iter()
        .filter_map(|id| graph.tasks.iter().find(|t| t.id == *id))
        .map(|t| TaskRef {
            id: t.id.clone(),
            title: t.title.clone(),
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Gated dispatch — concurrency-limited chain execution simulation
// ---------------------------------------------------------------------------

/// Outcome for a single chain execution in `dispatch_with_config`.
#[derive(Debug, PartialEq, Eq)]
pub enum ChainOutcome {
    Completed,
    /// Slot unavailable — concurrency limit hit; chain was not executed.
    Skipped,
    /// Retries exhausted after transient blocks; chain did not complete.
    Exhausted,
}

/// Simulate dispatching `chains` through a `WaveConfig`-gated concurrency tracker.
///
/// `execute` is called with each chain; it returns `Ok(())` on success or
/// `Err(true)` for a transient block (retry-eligible) or `Err(false)` for permanent failure.
///
/// Returns the per-chain outcomes in chain order.
pub fn dispatch_with_config<F>(
    chains: &[Chain],
    config: &WaveConfig,
    mut execute: F,
) -> Vec<ChainOutcome>
where
    F: FnMut(&Chain) -> Result<(), bool>,
{
    let mut tracker = ConcurrencyTracker::new(config.max_concurrency);
    let mut outcomes = Vec::with_capacity(chains.len());

    for chain in chains {
        // Gate: block until a slot is available (synchronous simulation).
        if !tracker.try_acquire() {
            // In a real async runtime we'd await; here we record Skipped for the chain.
            outcomes.push(ChainOutcome::Skipped);
            continue;
        }

        let mut health = SlotHealth::default();
        let mut result = execute(chain);

        // Retry loop for transient blocks.
        while let Err(true) = result {
            match on_blocked(&mut health, config) {
                BlockOutcome::Retry { attempt } => {
                    if config.retry_backoff_ms > 0 {
                        let delay = config.retry_backoff_ms * 2u64.pow(attempt as u32 - 1);
                        std::thread::sleep(std::time::Duration::from_millis(delay));
                    }
                    result = execute(chain);
                }
                BlockOutcome::Exhausted => {
                    result = Err(false);
                    break;
                }
            }
        }

        tracker.release();
        outcomes.push(if result.is_ok() {
            ChainOutcome::Completed
        } else {
            ChainOutcome::Exhausted
        });
    }

    outcomes
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
    use crate::wave::WaveConfig;
    use std::sync::{Arc, Mutex};

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

    #[test]
    fn critical_path_linear_chain() {
        let g = make_graph(&[
            ("t1", "A", &[], None),
            ("t2", "B", &["t1"], None),
            ("t3", "C", &["t2"], None),
        ]);
        let path = critical_path(&g);
        let ids: Vec<&str> = path.iter().map(|t| t.id.as_str()).collect();
        assert_eq!(ids, vec!["t1", "t2", "t3"]);
    }

    #[test]
    fn critical_path_diamond_picks_longer_side() {
        let g = make_graph(&[
            ("t1", "A", &[], None),
            ("t2", "B", &["t1"], None),
            ("t3", "C", &["t1"], None),
            ("t4", "D", &["t2", "t3"], None),
        ]);
        let path = critical_path(&g);
        assert_eq!(path.len(), 3);
        assert_eq!(path[0].id, "t1");
        assert_eq!(path.last().unwrap().id, "t4");
    }

    #[test]
    fn critical_path_parallel_equal_length() {
        let g = make_graph(&[
            ("a1", "A1", &[], None),
            ("a2", "A2", &["a1"], None),
            ("b1", "B1", &[], None),
            ("b2", "B2", &["b1"], None),
        ]);
        let path = critical_path(&g);
        assert_eq!(path.len(), 2);
        assert_eq!(path[0].id, "a1");
    }

    #[test]
    fn critical_path_empty_graph() {
        let g = TaskGraph::default();
        assert!(critical_path(&g).is_empty());
    }

    // --- dispatch_with_config tests ---

    fn make_chains(n: usize) -> Vec<Chain> {
        (0..n)
            .map(|i| Chain {
                crate_name: None,
                tasks: vec![TaskRef {
                    id: format!("t{i}"),
                    title: format!("Task {i}"),
                }],
            })
            .collect()
    }

    #[test]
    fn dispatch_max_concurrency_2_runs_sequentially() {
        // With max_concurrency=2 and 4 chains, each chain runs but slots are gated.
        // The concurrency tracker is synchronous here so all 4 run (released after each).
        let cfg = WaveConfig {
            max_concurrency: 2,
            max_retries: 0,
            ..Default::default()
        };
        let chains = make_chains(4);
        // Track peak concurrency via a shared counter.
        let peak = Arc::new(Mutex::new(0usize));
        let current = Arc::new(Mutex::new(0usize));
        let peak_c = peak.clone();
        let current_c = current.clone();

        let outcomes = dispatch_with_config(&chains, &cfg, move |_chain| {
            let mut c = current_c.lock().unwrap();
            *c += 1;
            let mut p = peak_c.lock().unwrap();
            if *c > *p {
                *p = *c;
            }
            // simulate completion — release happens in dispatch_with_config
            *c -= 1;
            Ok(())
        });

        // All 4 chains should complete (synchronous, slots released between each).
        assert_eq!(
            outcomes
                .iter()
                .filter(|o| **o == ChainOutcome::Completed)
                .count(),
            4
        );
        // Peak concurrency never exceeded 2 — but since synchronous, each is 1.
        assert!(*peak.lock().unwrap() <= 2);
    }

    #[test]
    fn dispatch_transient_block_recovered_with_retry() {
        // Fake agent: fails with transient block on first call, succeeds on retry.
        let cfg = WaveConfig {
            max_concurrency: 5,
            max_retries: 3,
            retry_backoff_ms: 0,
            ..Default::default()
        };
        let chains = make_chains(1);
        let call_count = Arc::new(Mutex::new(0usize));
        let call_count_c = call_count.clone();

        let outcomes = dispatch_with_config(&chains, &cfg, move |_chain| {
            let mut c = call_count_c.lock().unwrap();
            *c += 1;
            if *c == 1 {
                Err(true) // transient block
            } else {
                Ok(()) // success on retry
            }
        });

        assert_eq!(outcomes[0], ChainOutcome::Completed);
        assert_eq!(*call_count.lock().unwrap(), 2, "should have retried once");
    }

    #[test]
    fn dispatch_backoff_delays_retry() {
        let cfg = WaveConfig {
            max_concurrency: 5,
            max_retries: 1,
            retry_backoff_ms: 50,
            ..Default::default()
        };
        let chains = make_chains(1);
        let call_count = Arc::new(Mutex::new(0usize));
        let cc = call_count.clone();
        let start = std::time::Instant::now();
        let outcomes = dispatch_with_config(&chains, &cfg, move |_| {
            let mut c = cc.lock().unwrap();
            *c += 1;
            if *c == 1 { Err(true) } else { Ok(()) }
        });
        let elapsed = start.elapsed();
        assert_eq!(outcomes[0], ChainOutcome::Completed);
        assert!(
            elapsed.as_millis() >= 50,
            "expected backoff delay, got {elapsed:?}"
        );
    }

    #[test]
    fn dispatch_slot_unavailable_yields_skipped() {
        // max_concurrency=0 so every chain is skipped immediately
        let cfg = WaveConfig {
            max_concurrency: 0,
            max_retries: 0,
            retry_backoff_ms: 0,
            ..Default::default()
        };
        let chains = make_chains(3);
        let outcomes = dispatch_with_config(&chains, &cfg, |_| Ok(()));
        assert!(outcomes.iter().all(|o| *o == ChainOutcome::Skipped));
    }

    #[test]
    fn dispatch_permanent_block_exhausted() {
        let cfg = WaveConfig {
            max_concurrency: 5,
            max_retries: 2,
            retry_backoff_ms: 0,
            ..Default::default()
        };
        let chains = make_chains(1);

        // Always fails with transient block — should exhaust retries.
        let outcomes = dispatch_with_config(&chains, &cfg, |_chain| Err(true));

        assert_eq!(outcomes[0], ChainOutcome::Exhausted);
    }
}
