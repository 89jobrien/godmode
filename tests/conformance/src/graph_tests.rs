//! Conformance tests for godmode_core::graph — task state machine.

use godmode_core::graph;
use godmode_core::model::{Status, Task, TaskGraph};

use crate::harness::{ConformanceTest, TestCategory, TestContext, TestResult};

fn graph_with_chain() -> TaskGraph {
    let mut g = TaskGraph::default();
    g.tasks.push(Task::new("t1", "First"));
    let mut t2 = Task::new("t2", "Second");
    t2.depends_on = vec!["t1".into()];
    g.tasks.push(t2);
    g
}

// ---------------------------------------------------------------------------
// Runnable resolution
// ---------------------------------------------------------------------------

pub struct RunnableRootOnly;
impl ConformanceTest for RunnableRootOnly {
    fn name(&self) -> &str {
        "runnable_root_only"
    }
    fn crate_name(&self) -> &str {
        "graph"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Unit
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let g = graph_with_chain();
        let r = graph::runnable(&g);
        ctx.log_actual(
            "runnable_ids",
            &r.iter().map(|t| t.id.as_str()).collect::<Vec<_>>(),
        );
        ctx.assert_eq(
            &vec!["t1"],
            &r.iter().map(|t| t.id.as_str()).collect::<Vec<_>>(),
        );
        ctx.result()
    }
}

pub struct RunnableUnlocksAfterDep;
impl ConformanceTest for RunnableUnlocksAfterDep {
    fn name(&self) -> &str {
        "runnable_unlocks_after_dep_done"
    }
    fn crate_name(&self) -> &str {
        "graph"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Unit
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let mut g = graph_with_chain();
        graph::start(&mut g, "t1").unwrap();
        graph::complete(&mut g, "t1", None, None).unwrap();
        let r = graph::runnable(&g);
        ctx.assert_eq(
            &vec!["t2"],
            &r.iter().map(|t| t.id.as_str()).collect::<Vec<_>>(),
        );
        ctx.result()
    }
}

// ---------------------------------------------------------------------------
// State transitions
// ---------------------------------------------------------------------------

pub struct StartFailsUnmetDeps;
impl ConformanceTest for StartFailsUnmetDeps {
    fn name(&self) -> &str {
        "start_fails_unmet_deps"
    }
    fn crate_name(&self) -> &str {
        "graph"
    }
    fn category(&self) -> TestCategory {
        TestCategory::EdgeCase
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let mut g = graph_with_chain();
        let err = graph::start(&mut g, "t2").unwrap_err();
        if !err.to_string().contains("unmet dependencies") {
            ctx.fail(&format!("expected 'unmet dependencies', got: {}", err));
        }
        ctx.result()
    }
}

pub struct CompleteRequiresRunning;
impl ConformanceTest for CompleteRequiresRunning {
    fn name(&self) -> &str {
        "complete_requires_running"
    }
    fn crate_name(&self) -> &str {
        "graph"
    }
    fn category(&self) -> TestCategory {
        TestCategory::EdgeCase
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let mut g = graph_with_chain();
        let err = graph::complete(&mut g, "t1", None, None).unwrap_err();
        if !err.to_string().contains("pending") {
            ctx.fail(&format!("expected 'pending' in error, got: {}", err));
        }
        ctx.result()
    }
}

pub struct BlockSetsStatusAndNotes;
impl ConformanceTest for BlockSetsStatusAndNotes {
    fn name(&self) -> &str {
        "block_sets_status_and_notes"
    }
    fn crate_name(&self) -> &str {
        "graph"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Unit
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let mut g = graph_with_chain();
        graph::start(&mut g, "t1").unwrap();
        graph::block(&mut g, "t1", "three attempts failed").unwrap();
        ctx.assert_eq(&Status::Blocked, &g.tasks[0].status);
        ctx.assert_str_eq("three attempts failed", &g.tasks[0].notes);
        ctx.result()
    }
}

pub struct UnblockAllResetsBlocked;
impl ConformanceTest for UnblockAllResetsBlocked {
    fn name(&self) -> &str {
        "unblock_all_resets_blocked"
    }
    fn crate_name(&self) -> &str {
        "graph"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Unit
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let mut g = TaskGraph::default();
        let mut t1 = Task::new("t1", "A");
        t1.status = Status::Blocked;
        t1.notes = "reason".into();
        let mut t2 = Task::new("t2", "B");
        t2.status = Status::Done;
        g.tasks.push(t1);
        g.tasks.push(t2);
        let count = graph::unblock_all(&mut g);
        ctx.assert_eq(&1usize, &count);
        ctx.assert_eq(&Status::Pending, &g.tasks[0].status);
        ctx.assert_eq(&Status::Done, &g.tasks[1].status);
        ctx.result()
    }
}

// ---------------------------------------------------------------------------
// Cycle detection
// ---------------------------------------------------------------------------

pub struct CycleDetectedSelfLoop;
impl ConformanceTest for CycleDetectedSelfLoop {
    fn name(&self) -> &str {
        "cycle_detected_self_loop"
    }
    fn crate_name(&self) -> &str {
        "graph"
    }
    fn category(&self) -> TestCategory {
        TestCategory::EdgeCase
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let mut g = TaskGraph::default();
        let mut t = Task::new("t1", "A");
        t.depends_on = vec!["t1".into()];
        let err = graph::add(&mut g, t).unwrap_err();
        if !err.to_string().contains("cycle") {
            ctx.fail(&format!("expected cycle error, got: {}", err));
        }
        ctx.result()
    }
}

pub struct CycleDetectedTransitive;
impl ConformanceTest for CycleDetectedTransitive {
    fn name(&self) -> &str {
        "cycle_detected_transitive"
    }
    fn crate_name(&self) -> &str {
        "graph"
    }
    fn category(&self) -> TestCategory {
        TestCategory::EdgeCase
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let mut g = TaskGraph::default();
        let mut b = Task::new("b", "B");
        b.depends_on = vec!["a".into()];
        graph::add(&mut g, b).unwrap();
        let mut a = Task::new("a", "A");
        a.depends_on = vec!["b".into()];
        let err = graph::add(&mut g, a).unwrap_err();
        if !err.to_string().contains("cycle") {
            ctx.fail(&format!("expected cycle error, got: {}", err));
        }
        ctx.result()
    }
}

pub struct DiamondDagNoCycle;
impl ConformanceTest for DiamondDagNoCycle {
    fn name(&self) -> &str {
        "diamond_dag_no_cycle"
    }
    fn crate_name(&self) -> &str {
        "graph"
    }
    fn category(&self) -> TestCategory {
        TestCategory::EdgeCase
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let mut g = TaskGraph::default();
        graph::add(&mut g, Task::new("t1", "A")).unwrap();
        let mut t2 = Task::new("t2", "B");
        t2.depends_on = vec!["t1".into()];
        graph::add(&mut g, t2).unwrap();
        let mut t3 = Task::new("t3", "C");
        t3.depends_on = vec!["t1".into()];
        graph::add(&mut g, t3).unwrap();
        ctx.assert_eq(&3usize, &g.tasks.len());
        ctx.result()
    }
}

// ---------------------------------------------------------------------------
// ID generation
// ---------------------------------------------------------------------------

pub struct NextTaskIdSequential;
impl ConformanceTest for NextTaskIdSequential {
    fn name(&self) -> &str {
        "next_task_id_sequential"
    }
    fn crate_name(&self) -> &str {
        "graph"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Unit
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let mut g = TaskGraph::default();
        ctx.assert_str_eq("t1", &graph::next_task_id(&g));
        graph::add(&mut g, Task::new("t1", "A")).unwrap();
        ctx.assert_str_eq("t2", &graph::next_task_id(&g));
        graph::add(&mut g, Task::new("t3", "C")).unwrap(); // gap
        ctx.assert_str_eq("t2", &graph::next_task_id(&g)); // fills gap
        ctx.result()
    }
}

// ---------------------------------------------------------------------------
// Clear / remove
// ---------------------------------------------------------------------------

pub struct ClearDoneOnly;
impl ConformanceTest for ClearDoneOnly {
    fn name(&self) -> &str {
        "clear_done_only"
    }
    fn crate_name(&self) -> &str {
        "graph"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Unit
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let mut g = TaskGraph::default();
        let mut t1 = Task::new("t1", "A");
        t1.status = Status::Done;
        g.tasks.push(t1);
        g.tasks.push(Task::new("t2", "B"));
        let removed = graph::clear(&mut g, true);
        ctx.assert_eq(&1usize, &removed);
        ctx.assert_eq(&1usize, &g.tasks.len());
        ctx.assert_str_eq("t2", &g.tasks[0].id);
        ctx.result()
    }
}

/// Register all graph conformance tests.
pub fn all() -> Vec<Box<dyn ConformanceTest>> {
    vec![
        Box::new(RunnableRootOnly),
        Box::new(RunnableUnlocksAfterDep),
        Box::new(StartFailsUnmetDeps),
        Box::new(CompleteRequiresRunning),
        Box::new(BlockSetsStatusAndNotes),
        Box::new(UnblockAllResetsBlocked),
        Box::new(CycleDetectedSelfLoop),
        Box::new(CycleDetectedTransitive),
        Box::new(DiamondDagNoCycle),
        Box::new(NextTaskIdSequential),
        Box::new(ClearDoneOnly),
    ]
}
