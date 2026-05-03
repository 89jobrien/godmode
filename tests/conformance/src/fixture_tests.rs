//! Fixture-driven conformance tests — expected outputs loaded from
//! `tests/conformance/fixtures/expected/*.json`.

use godmode_core::{graph, model::Task, model::TaskGraph, plan};

use crate::harness::{
    ConformanceTest, TestCategory, TestContext, TestResult, fixtures::FixtureLoader,
};

pub struct FixtureGraphRunnable;
impl ConformanceTest for FixtureGraphRunnable {
    fn name(&self) -> &str {
        "fixture_graph_runnable"
    }
    fn crate_name(&self) -> &str {
        "graph"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Integration
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let loader = FixtureLoader::new();
        let fixture = match loader.load("graph_runnable") {
            Ok(f) => f,
            Err(e) => {
                ctx.fail(&format!("fixture load failed: {}", e));
                return ctx.result();
            }
        };

        let mut g = TaskGraph::default();
        g.tasks.push(Task::new("t1", "First"));
        let mut t2 = Task::new("t2", "Second");
        t2.depends_on = vec!["t1".into()];
        g.tasks.push(t2);

        // Check initial runnable matches fixture
        let initial_expected = fixture.str_array("initial_runnable");
        let initial_actual: Vec<&str> = graph::runnable(&g).iter().map(|t| t.id.as_str()).collect();
        ctx.log_expected("initial_runnable", &initial_expected);
        ctx.log_actual("initial_runnable", &initial_actual);
        ctx.assert_eq(&initial_expected, &initial_actual);

        // Advance t1 to done, check t2 unlocks
        graph::start(&mut g, "t1").unwrap();
        graph::complete(&mut g, "t1", None, None).unwrap();
        let after_expected = fixture.str_array("after_t1_done_runnable");
        let after_actual: Vec<&str> = graph::runnable(&g).iter().map(|t| t.id.as_str()).collect();
        ctx.log_expected("after_t1_done_runnable", &after_expected);
        ctx.log_actual("after_t1_done_runnable", &after_actual);
        ctx.assert_eq(&after_expected, &after_actual);

        ctx.result()
    }
}

pub struct FixturePlanParse;
impl ConformanceTest for FixturePlanParse {
    fn name(&self) -> &str {
        "fixture_plan_parse"
    }
    fn crate_name(&self) -> &str {
        "plan"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Integration
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let loader = FixtureLoader::new();
        let fixture = match loader.load("plan_parse") {
            Ok(f) => f,
            Err(e) => {
                ctx.fail(&format!("fixture load failed: {}", e));
                return ctx.result();
            }
        };

        let input = match fixture.str_field("input") {
            Some(s) => s,
            None => {
                ctx.fail("fixture missing 'input' field");
                return ctx.result();
            }
        };

        let tasks = match plan::parse(input) {
            Ok(t) => t,
            Err(e) => {
                ctx.fail(&format!("plan::parse failed: {}", e));
                return ctx.result();
            }
        };

        let expected_ids = fixture.str_array("expected_ids");
        let actual_ids: Vec<&str> = tasks.iter().map(|t| t.id.as_str()).collect();
        ctx.assert_eq(&expected_ids, &actual_ids);

        let expected_titles = fixture.str_array("expected_titles");
        let actual_titles: Vec<&str> = tasks.iter().map(|t| t.title.as_str()).collect();
        ctx.assert_eq(&expected_titles, &actual_titles);

        // Check deps array
        if let Some(deps_arr) = fixture.get("expected_deps").and_then(|v| v.as_array()) {
            for (i, expected_deps) in deps_arr.iter().enumerate() {
                let exp: Vec<&str> = expected_deps
                    .as_array()
                    .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
                    .unwrap_or_default();
                let act: Vec<&str> = tasks[i].depends_on.iter().map(|s| s.as_str()).collect();
                ctx.assert_eq(&exp, &act);
            }
        }

        ctx.result()
    }
}

pub fn all() -> Vec<Box<dyn ConformanceTest>> {
    vec![Box::new(FixtureGraphRunnable), Box::new(FixturePlanParse)]
}
