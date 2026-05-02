# Test Dimension Examples

Concrete Rust patterns for each of the five dimensions.

## Unit

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_task_with_duplicate_id_returns_err() {
        let mut g = TaskGraph::default();
        let t = Task::new("t1".into(), "First".into());
        g.add(t.clone()).unwrap();
        assert!(g.add(t).is_err());
    }
}
```

## Property

```rust
use proptest::prelude::*;

prop_compose! {
    fn arb_task_id()(s in "[a-z][a-z0-9-]{0,15}") -> String { s }
}

prop_compose! {
    fn arb_task()(id in arb_task_id(), title in ".*") -> Task {
        Task::new(id, title)
    }
}

proptest! {
    #[test]
    fn add_then_remove_leaves_graph_unchanged(task in arb_task()) {
        let mut g = TaskGraph::default();
        let id = task.id.clone();
        g.add(task).unwrap();
        g.remove(&id).unwrap();
        prop_assert!(g.tasks.is_empty());
    }

    #[test]
    fn runnable_tasks_have_no_unmet_deps(tasks in prop::collection::vec(arb_task(), 1..10)) {
        let mut g = TaskGraph::default();
        for t in tasks { let _ = g.add(t); }
        for t in g.runnable() {
            prop_assert!(t.depends_on.is_empty() ||
                t.depends_on.iter().all(|d| g.tasks.iter().any(|x| &x.id == d && x.status == Status::Done)));
        }
    }
}
```

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
    graph::add(&mut g, Task::new("t1".into(), "One".into())).unwrap();
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
    let mut t = Task::new("t1".into(), "Self".into());
    t.depends_on = vec!["t1".into()]; // self-referential
    g.add(t).unwrap();
    // must not panic
    let _ = g.runnable();
}
```

For proptest regressions, the `proptest-regressions/` directory is committed automatically
when a counterexample is found. Never delete these files.
