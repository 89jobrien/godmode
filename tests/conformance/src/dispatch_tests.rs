//! Conformance tests for godmode_core::dispatch — parallel chain assignment.

use godmode_core::dispatch;
use godmode_core::model::{Task, TaskGraph};

use crate::harness::{ConformanceTest, TestCategory, TestContext, TestResult};

/// Verifies that independent tasks remain available for parallel dispatch.
pub struct DispatchIndependentTasksParallel;
impl ConformanceTest for DispatchIndependentTasksParallel {
    fn name(&self) -> &str {
        "dispatch_independent_tasks_parallel"
    }
    fn crate_name(&self) -> &str {
        "dispatch"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Unit
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let mut g = TaskGraph::default();
        g.tasks.push(Task::new("t1", "A"));
        g.tasks.push(Task::new("t2", "B"));
        g.tasks.push(Task::new("t3", "C"));
        let chains = dispatch::independent_chains(&g, 5);
        ctx.log_actual("chain_count", &chains.len());
        let total: usize = chains.iter().map(|c| c.tasks.len()).sum();
        ctx.assert_eq(&3usize, &total);
        ctx.result()
    }
}

/// Verifies that dependent tasks are dispatched in one ordered chain.
pub struct DispatchChainedTasksOneChain;
impl ConformanceTest for DispatchChainedTasksOneChain {
    fn name(&self) -> &str {
        "dispatch_chained_tasks_one_chain"
    }
    fn crate_name(&self) -> &str {
        "dispatch"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Unit
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let mut g = TaskGraph::default();
        g.tasks.push(Task::new("t1", "A"));
        let mut t2 = Task::new("t2", "B");
        t2.depends_on = vec!["t1".into()];
        g.tasks.push(t2);
        let chains = dispatch::independent_chains(&g, 5);
        let total: usize = chains.iter().map(|c| c.tasks.len()).sum();
        ctx.assert_eq(&2usize, &total);
        // t1→t2 chain must appear in the same chain, in order
        let has_ordered = chains.iter().any(|c| {
            let ids: Vec<_> = c.tasks.iter().map(|t| t.id.as_str()).collect();
            ids == vec!["t1", "t2"]
        });
        if !has_ordered {
            ctx.fail("chained tasks t1→t2 must appear in the same chain in order");
        }
        ctx.result()
    }
}

/// Verifies that dispatch does not exceed the requested chain limit.
pub struct DispatchRespectsMaxChains;
impl ConformanceTest for DispatchRespectsMaxChains {
    fn name(&self) -> &str {
        "dispatch_respects_max_chains"
    }
    fn crate_name(&self) -> &str {
        "dispatch"
    }
    fn category(&self) -> TestCategory {
        TestCategory::EdgeCase
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let mut g = TaskGraph::default();
        for i in 1..=10u32 {
            g.tasks
                .push(Task::new(format!("t{}", i), format!("Task {}", i)));
        }
        let max = 3;
        let chains = dispatch::independent_chains(&g, max);
        ctx.log_actual("chain_count", &chains.len());
        if chains.len() > max {
            ctx.fail(&format!("got {} chains, max is {}", chains.len(), max));
        }
        ctx.result()
    }
}

/// Verifies that dispatching an empty graph produces no chains.
pub struct DispatchEmptyGraphEmptyChains;
impl ConformanceTest for DispatchEmptyGraphEmptyChains {
    fn name(&self) -> &str {
        "dispatch_empty_graph"
    }
    fn crate_name(&self) -> &str {
        "dispatch"
    }
    fn category(&self) -> TestCategory {
        TestCategory::EdgeCase
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let g = TaskGraph::default();
        let chains = dispatch::independent_chains(&g, 5);
        ctx.assert_eq(&0usize, &chains.len());
        ctx.result()
    }
}

/// Returns all dispatch conformance tests.
pub fn all() -> Vec<Box<dyn ConformanceTest>> {
    vec![
        Box::new(DispatchIndependentTasksParallel),
        Box::new(DispatchChainedTasksOneChain),
        Box::new(DispatchRespectsMaxChains),
        Box::new(DispatchEmptyGraphEmptyChains),
    ]
}
