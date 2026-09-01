//! Criterion benchmarks for godmode-core hot paths.
//!
//! Run: cargo bench -p godmode-conformance

// Criterion generates an internal public harness function without rustdoc.
#![allow(missing_docs)]

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use godmode_core::{
    dispatch, graph,
    model::{Status, Task, TaskGraph},
    plan,
};

// ── graph ──────────────────────────────────────────────────────────────────

fn bench_runnable(c: &mut Criterion) {
    let mut group = c.benchmark_group("graph/runnable");
    for n in [10, 50, 200] {
        let g = linear_chain(n);
        group.bench_with_input(BenchmarkId::from_parameter(n), &g, |b, g| {
            b.iter(|| graph::runnable(g));
        });
    }
    group.finish();
}

fn bench_graph_add(c: &mut Criterion) {
    let mut group = c.benchmark_group("graph/add");
    for n in [10, 50, 100] {
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| {
                let mut g = TaskGraph::default();
                for i in 0..n {
                    graph::add(&mut g, Task::new(format!("t{}", i), "x")).unwrap();
                }
                g
            });
        });
    }
    group.finish();
}

fn bench_unblock_all(c: &mut Criterion) {
    let mut group = c.benchmark_group("graph/unblock_all");
    for n in [10, 100, 500] {
        let mut g = TaskGraph::default();
        for i in 0..n {
            let mut t = Task::new(format!("t{}", i), "x");
            t.status = Status::Blocked;
            g.tasks.push(t);
        }
        group.bench_with_input(BenchmarkId::from_parameter(n), &g, |b, g| {
            b.iter(|| {
                let mut g2 = g.clone();
                graph::unblock_all(&mut g2)
            });
        });
    }
    group.finish();
}

// ── plan ──────────────────────────────────────────────────────────────────

fn bench_plan_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("plan/parse");
    for n in [5, 20, 100] {
        let md: String = (1..=n)
            .map(|i| format!("### Task {}: Task {}\n", i, i))
            .collect();
        group.bench_with_input(BenchmarkId::from_parameter(n), &md, |b, md| {
            b.iter(|| plan::parse(md).unwrap());
        });
    }
    group.finish();
}

// ── dispatch ──────────────────────────────────────────────────────────────

fn bench_dispatch(c: &mut Criterion) {
    let mut group = c.benchmark_group("dispatch/independent_chains");
    for n in [10, 50, 200] {
        let g = flat_graph(n);
        group.bench_with_input(BenchmarkId::from_parameter(n), &g, |b, g| {
            b.iter(|| dispatch::independent_chains(g, 5));
        });
    }
    group.finish();
}

// ── helpers ───────────────────────────────────────────────────────────────

fn linear_chain(n: usize) -> TaskGraph {
    let mut g = TaskGraph::default();
    for i in 0..n {
        let mut t = Task::new(format!("t{}", i), "x");
        if i > 0 {
            t.depends_on = vec![format!("t{}", i - 1)];
        }
        g.tasks.push(t);
    }
    g
}

fn flat_graph(n: usize) -> TaskGraph {
    let mut g = TaskGraph::default();
    for i in 0..n {
        g.tasks.push(Task::new(format!("t{}", i), "x"));
    }
    g
}

criterion_group!(
    benches,
    bench_runnable,
    bench_graph_add,
    bench_unblock_all,
    bench_plan_parse,
    bench_dispatch,
);
criterion_main!(benches);
