//! Conformance tests for godmode_core::plan — markdown plan parsing.

use godmode_core::model::Status;
use godmode_core::plan;

use crate::harness::{ConformanceTest, TestCategory, TestContext, TestResult};

/// Verifies that task headings produce tasks with the expected IDs and titles.
pub struct PlanParsesTaskHeadings;
impl ConformanceTest for PlanParsesTaskHeadings {
    fn name(&self) -> &str {
        "plan_parses_task_headings"
    }
    fn crate_name(&self) -> &str {
        "plan"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Unit
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let md = "### Task 1: Write tests\n### Task 2: Implement\n";
        let tasks = match plan::parse(md) {
            Ok(t) => t,
            Err(e) => {
                ctx.fail(&e.to_string());
                return ctx.result();
            }
        };
        ctx.assert_eq(&2usize, &tasks.len());
        ctx.assert_str_eq("t1", &tasks[0].id);
        ctx.assert_str_eq("Write tests", &tasks[0].title);
        ctx.assert_str_eq("t2", &tasks[1].id);
        ctx.result()
    }
}

/// Verifies that parsed plan tasks receive sequential dependencies.
pub struct PlanSequentialDeps;
impl ConformanceTest for PlanSequentialDeps {
    fn name(&self) -> &str {
        "plan_sequential_deps"
    }
    fn crate_name(&self) -> &str {
        "plan"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Unit
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let md = "### Task 1: A\n### Task 2: B\n### Task 3: C\n";
        let tasks = match plan::parse(md) {
            Ok(t) => t,
            Err(e) => {
                ctx.fail(&e.to_string());
                return ctx.result();
            }
        };
        ctx.assert_eq(&3usize, &tasks.len());
        ctx.assert_eq(&Vec::<String>::new(), &tasks[0].depends_on);
        ctx.assert_eq(&vec!["t1".to_string()], &tasks[1].depends_on);
        ctx.assert_eq(&vec!["t2".to_string()], &tasks[2].depends_on);
        ctx.result()
    }
}

/// Verifies that parsed plan tasks begin in the pending state.
pub struct PlanAllPendingStatus;
impl ConformanceTest for PlanAllPendingStatus {
    fn name(&self) -> &str {
        "plan_all_pending_status"
    }
    fn crate_name(&self) -> &str {
        "plan"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Unit
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let md = "### Task 1: A\n### Task 2: B\n";
        let tasks = match plan::parse(md) {
            Ok(t) => t,
            Err(e) => {
                ctx.fail(&e.to_string());
                return ctx.result();
            }
        };
        for t in &tasks {
            if t.status != Status::Pending {
                ctx.fail(&format!(
                    "task {} has status {:?}, expected Pending",
                    t.id, t.status
                ));
            }
        }
        ctx.result()
    }
}

/// Verifies that an empty plan produces no tasks.
pub struct PlanEmptyInputEmptyOutput;
impl ConformanceTest for PlanEmptyInputEmptyOutput {
    fn name(&self) -> &str {
        "plan_empty_input"
    }
    fn crate_name(&self) -> &str {
        "plan"
    }
    fn category(&self) -> TestCategory {
        TestCategory::EdgeCase
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let tasks = match plan::parse("") {
            Ok(t) => t,
            Err(e) => {
                ctx.fail(&e.to_string());
                return ctx.result();
            }
        };
        ctx.assert_eq(&0usize, &tasks.len());
        ctx.result()
    }
}

/// Verifies that non-task headings and prose do not produce tasks.
pub struct PlanIgnoresNonHeadings;
impl ConformanceTest for PlanIgnoresNonHeadings {
    fn name(&self) -> &str {
        "plan_ignores_non_headings"
    }
    fn crate_name(&self) -> &str {
        "plan"
    }
    fn category(&self) -> TestCategory {
        TestCategory::EdgeCase
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let md = "# Title\n\nSome prose.\n\n## Section\n\n### Task 1: Real task\n\nBody text.\n";
        let tasks = match plan::parse(md) {
            Ok(t) => t,
            Err(e) => {
                ctx.fail(&e.to_string());
                return ctx.result();
            }
        };
        ctx.assert_eq(&1usize, &tasks.len());
        ctx.assert_str_eq("Real task", &tasks[0].title);
        ctx.result()
    }
}

/// Returns all plan parser conformance tests.
pub fn all() -> Vec<Box<dyn ConformanceTest>> {
    vec![
        Box::new(PlanParsesTaskHeadings),
        Box::new(PlanSequentialDeps),
        Box::new(PlanAllPendingStatus),
        Box::new(PlanEmptyInputEmptyOutput),
        Box::new(PlanIgnoresNonHeadings),
    ]
}
