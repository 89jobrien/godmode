//! Conformance tests for godmode_core::integrations::cruxx — trace event emission.

use godmode_core::integrations::cruxx;
use slashcrux::StepState;

use crate::harness::{ConformanceTest, TestCategory, TestContext, TestResult};

pub struct CruxxStartedEmitsRunning;
impl ConformanceTest for CruxxStartedEmitsRunning {
    fn name(&self) -> &str {
        "cruxx_started_emits_running"
    }
    fn crate_name(&self) -> &str {
        "cruxx"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Unit
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let dir = tempfile::TempDir::new().unwrap();
        let event = cruxx::TaskEvent::started("t1", "Write test");
        cruxx::append_event(dir.path(), &event).unwrap();

        let content = std::fs::read_to_string(cruxx::trace_file(dir.path())).unwrap();
        let line: serde_json::Value = serde_json::from_str(content.trim()).unwrap();
        ctx.assert_str_eq("t1", line["step_name"].as_str().unwrap_or(""));
        ctx.assert_str_eq("running", line["state"].as_str().unwrap_or(""));
        ctx.assert_str_eq("Write test", line["title"].as_str().unwrap_or(""));
        if line.get("ts").is_none() {
            ctx.fail("missing ts field");
        }
        ctx.result()
    }
}

pub struct CruxxCompletedEmitsState;
impl ConformanceTest for CruxxCompletedEmitsState {
    fn name(&self) -> &str {
        "cruxx_completed_emits_state"
    }
    fn crate_name(&self) -> &str {
        "cruxx"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Unit
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let dir = tempfile::TempDir::new().unwrap();
        let event = cruxx::TaskEvent::completed(
            "t2",
            "Impl",
            Some("deadbeef".into()),
            Some("all green".into()),
        );
        cruxx::append_event(dir.path(), &event).unwrap();

        let content = std::fs::read_to_string(cruxx::trace_file(dir.path())).unwrap();
        let line: serde_json::Value = serde_json::from_str(content.trim()).unwrap();
        ctx.assert_str_eq("completed", line["state"].as_str().unwrap_or(""));
        ctx.assert_str_eq("deadbeef", line["commit"].as_str().unwrap_or(""));
        ctx.assert_str_eq("all green", line["notes"].as_str().unwrap_or(""));
        ctx.result()
    }
}

pub struct CruxxBlockedEmitsCancelled;
impl ConformanceTest for CruxxBlockedEmitsCancelled {
    fn name(&self) -> &str {
        "cruxx_blocked_emits_cancelled"
    }
    fn crate_name(&self) -> &str {
        "cruxx"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Unit
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let dir = tempfile::TempDir::new().unwrap();
        let event = cruxx::TaskEvent::blocked("t3", "Stuck", Some("three attempts".into()));
        cruxx::append_event(dir.path(), &event).unwrap();

        let content = std::fs::read_to_string(cruxx::trace_file(dir.path())).unwrap();
        let line: serde_json::Value = serde_json::from_str(content.trim()).unwrap();
        // blocked = externally stopped, not internally failed → "cancelled"
        ctx.assert_str_eq("cancelled", line["state"].as_str().unwrap_or(""));
        ctx.assert_str_eq("three attempts", line["reason"].as_str().unwrap_or(""));
        ctx.result()
    }
}

pub struct CruxxPendingEmitsPending;
impl ConformanceTest for CruxxPendingEmitsPending {
    fn name(&self) -> &str {
        "cruxx_pending_emits_pending"
    }
    fn crate_name(&self) -> &str {
        "cruxx"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Unit
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let dir = tempfile::TempDir::new().unwrap();
        let event = cruxx::TaskEvent::pending("t0", "New task");
        cruxx::append_event(dir.path(), &event).unwrap();

        let content = std::fs::read_to_string(cruxx::trace_file(dir.path())).unwrap();
        let line: serde_json::Value = serde_json::from_str(content.trim()).unwrap();
        ctx.assert_str_eq("pending", line["state"].as_str().unwrap_or(""));
        ctx.assert_str_eq("t0", line["step_name"].as_str().unwrap_or(""));
        ctx.result()
    }
}

pub struct CruxxStateIsSlashcruxVocabulary;
impl ConformanceTest for CruxxStateIsSlashcruxVocabulary {
    fn name(&self) -> &str {
        "cruxx_state_is_slashcrux_vocabulary"
    }
    fn crate_name(&self) -> &str {
        "cruxx"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Integration
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        // Verify TaskEvent.state field is a real slashcrux::StepState, not a bespoke enum.
        let e = cruxx::TaskEvent::started("t1", "x");
        ctx.assert_eq(&StepState::Running, &e.state);
        let e = cruxx::TaskEvent::completed("t2", "x", None, None);
        ctx.assert_eq(&StepState::Completed, &e.state);
        let e = cruxx::TaskEvent::pending("t0", "x");
        ctx.assert_eq(&StepState::Pending, &e.state);
        let e = cruxx::TaskEvent::blocked("t3", "x", None);
        ctx.assert_eq(&StepState::Cancelled, &e.state);
        ctx.result()
    }
}

pub struct CruxxAppendsMultipleLines;
impl ConformanceTest for CruxxAppendsMultipleLines {
    fn name(&self) -> &str {
        "cruxx_appends_multiple_lines"
    }
    fn crate_name(&self) -> &str {
        "cruxx"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Unit
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let dir = tempfile::TempDir::new().unwrap();
        for id in ["t1", "t2", "t3"] {
            let event = cruxx::TaskEvent::started(id, "task");
            cruxx::append_event(dir.path(), &event).unwrap();
        }
        let content = std::fs::read_to_string(cruxx::trace_file(dir.path())).unwrap();
        let lines = content.lines().count();
        ctx.assert_eq(&3usize, &lines);
        ctx.result()
    }
}

pub fn all() -> Vec<Box<dyn ConformanceTest>> {
    vec![
        Box::new(CruxxStartedEmitsRunning),
        Box::new(CruxxCompletedEmitsState),
        Box::new(CruxxBlockedEmitsCancelled),
        Box::new(CruxxPendingEmitsPending),
        Box::new(CruxxStateIsSlashcruxVocabulary),
        Box::new(CruxxAppendsMultipleLines),
    ]
}
