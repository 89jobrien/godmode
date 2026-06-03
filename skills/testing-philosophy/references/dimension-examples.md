# Test Dimension Examples

Concrete Rust patterns for each of the seven testing dimensions.

## Unit

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_task_with_duplicate_id_returns_err() {
        let mut g = TaskGraph::default();
        graph::add(&mut g, Task::new("t1", "First")).unwrap();
        let err = graph::add(&mut g, Task::new("t1", "Duplicate"));
        assert!(err.is_err());
    }
}
```

## Property

```rust
use proptest::prelude::*;

fn task_id() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9]{0,7}".prop_map(|s| s)
}

fn task_title() -> impl Strategy<Value = String> {
    "[A-Za-z][A-Za-z0-9 ]{1,30}".prop_map(|s| s)
}

proptest! {
    #[test]
    fn add_then_remove_leaves_graph_unchanged(
        id in task_id(), title in task_title()
    ) {
        let mut g = TaskGraph::default();
        graph::add(&mut g, Task::new(id.clone(), title)).unwrap();
        graph::remove(&mut g, &id).unwrap();
        prop_assert!(g.tasks.is_empty());
    }

    #[test]
    fn runnable_tasks_have_no_unmet_deps(g in arb_dag(8)) {
        let done_ids: HashSet<&str> = g.tasks.iter()
            .filter(|t| t.status == Status::Done)
            .map(|t| t.id.as_str())
            .collect();
        for t in graph::runnable(&g) {
            for dep in &t.depends_on {
                prop_assert!(done_ids.contains(dep.as_str()));
            }
        }
    }
}
```

## Fuzz

```rust
// fuzz/fuzz_targets/fuzz_yaml_roundtrip.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use godmode_core::model::TaskGraph;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        if let Ok(graph) = serde_yaml::from_str::<TaskGraph>(s) {
            let _ = serde_yaml::to_string(&graph);
            // Assert invariant inside fuzz target:
            let s = graph.summary();
            assert_eq!(
                s.done + s.running + s.pending + s.blocked,
                graph.tasks.len()
            );
        }
    }
});
```

Run with: `cargo +nightly fuzz run fuzz_yaml_roundtrip -- -max_total_time=60`

## Model Check

```rust
#[cfg(kani)]
mod kani_proofs {
    use super::*;

    #[kani::proof]
    #[kani::unwind(5)]
    fn check_concurrency_tracker_never_exceeds_max() {
        let max: usize = kani::any();
        kani::assume(max > 0 && max <= 10);
        let mut tracker = ConcurrencyTracker::new(max);
        // Symbolic sequence of acquire/release
        for _ in 0..5 {
            let action: bool = kani::any();
            if action { tracker.try_acquire(); } else { tracker.release(); }
            assert!(tracker.active() <= max);
        }
    }
}
```

Run with: `cargo kani --harness check_concurrency_tracker_never_exceeds_max`

## Conformance

```rust
// tests/conformance_task_store.rs
// Reusable suite — call with any impl of TaskStore

fn assert_task_store_contract<S: TaskStore>(mut store: S) {
    // contract: add then get returns the same task
    let t = Task::new("c1".into(), "Conformance".into());
    store.add(t.clone()).unwrap();
    assert_eq!(store.get("c1").unwrap().title, t.title);

    // contract: add duplicate returns Err
    assert!(store.add(t).is_err());

    // contract: remove nonexistent returns Err
    assert!(store.remove("nonexistent").is_err());
}

#[test]
fn in_memory_store_satisfies_contract() {
    assert_task_store_contract(InMemoryTaskStore::default());
}

#[test]
fn yaml_store_satisfies_contract() {
    let dir = tempfile::tempdir().unwrap();
    assert_task_store_contract(YamlTaskStore::new(dir.path()));
}
```

## Integration

```rust
// tests/graph_integration.rs
// Tests the full load → mutate → save → reload cycle across the file boundary

#[test]
fn graph_round_trips_through_yaml() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    let mut g = graph::load(root).expect("load empty");
    graph::add(&mut g, Task::new("t1", "One")).unwrap();
    graph::save(root, &g).expect("save");

    let g2 = graph::load(root).expect("reload");
    assert_eq!(g2.tasks.len(), 1);
    assert_eq!(g2.tasks[0].id, "t1");
}
```

## Regression

```rust
// After a bug: graph::runnable panicked on tasks with self-referential deps

#[test]
fn runnable_does_not_panic_on_self_dep() {
    let mut g = TaskGraph::default();
    let mut t = Task::new("t1", "Self");
    t.depends_on = vec!["t1".into()]; // self-referential
    // graph::add rejects cycles — this is the expected guard
    assert!(graph::add(&mut g, t).is_err());
}
```

For proptest regressions, the `proptest-regressions/` directory is committed automatically
when a counterexample is found. Never delete these files.
