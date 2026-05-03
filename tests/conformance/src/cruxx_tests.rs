//! Conformance tests for godmode_core::integrations::cruxx — Step constructor API.

use chrono::Utc;
use cruxx_core::types::crux_value::Crux;
use cruxx_core::types::error::CruxErr;
use cruxx_core::types::step::{Step, StepKind, StepStatus};
use godmode_core::integrations::cruxx;
use godmode_core::model::TaskGraph;
use godmode_core::session_trace::Session;

fn make_step(name: &str) -> Step {
    Step {
        name: name.to_string(),
        kind: StepKind::Plain,
        status: StepStatus::Ok,
        confidence: 1.0,
        started_at: Utc::now(),
        duration_ms: 0,
        input_hash: 0,
        content_hash: None,
        output: None,
        error: None,
        attempt: 1,
        events: vec![],
    }
}

use crate::harness::{ConformanceTest, TestCategory, TestContext, TestResult};

pub struct CruxxStepStartedIsOk;
impl ConformanceTest for CruxxStepStartedIsOk {
    fn name(&self) -> &str {
        "cruxx_step_started_is_ok"
    }
    fn crate_name(&self) -> &str {
        "cruxx"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Unit
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let step = cruxx::step_started("t1");
        ctx.assert_str_eq("t1", &step.name);
        ctx.assert_eq(&StepStatus::Ok, &step.status);
        ctx.assert_eq(&1u32, &step.attempt);
        ctx.result()
    }
}

pub struct CruxxStepPendingIsSkipped;
impl ConformanceTest for CruxxStepPendingIsSkipped {
    fn name(&self) -> &str {
        "cruxx_step_pending_is_skipped"
    }
    fn crate_name(&self) -> &str {
        "cruxx"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Unit
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let step = cruxx::step_pending("t0");
        ctx.assert_str_eq("t0", &step.name);
        ctx.assert_eq(&StepStatus::Skipped, &step.status);
        ctx.assert_eq(&0u32, &step.attempt);
        ctx.result()
    }
}

pub struct CruxxStepCompletedHasOutput;
impl ConformanceTest for CruxxStepCompletedHasOutput {
    fn name(&self) -> &str {
        "cruxx_step_completed_has_output"
    }
    fn crate_name(&self) -> &str {
        "cruxx"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Unit
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let step = cruxx::step_completed("t2", Some("deadbeef"), Some("all green"));
        ctx.assert_eq(&StepStatus::Ok, &step.status);
        let output = step.output.as_ref().expect("output should be Some");
        ctx.assert_str_eq("deadbeef", output["commit"].as_str().unwrap_or(""));
        ctx.assert_str_eq("all green", output["notes"].as_str().unwrap_or(""));
        ctx.result()
    }
}

pub struct CruxxStepBlockedIsErr;
impl ConformanceTest for CruxxStepBlockedIsErr {
    fn name(&self) -> &str {
        "cruxx_step_blocked_is_err"
    }
    fn crate_name(&self) -> &str {
        "cruxx"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Unit
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let step = cruxx::step_blocked("t3", Some("external dep missing"));
        ctx.assert_eq(&StepStatus::Err, &step.status);
        ctx.assert_str_eq("external dep missing", step.error.as_deref().unwrap_or(""));
        ctx.result()
    }
}

pub struct CruxxStepsSerializeRoundtrip;
impl ConformanceTest for CruxxStepsSerializeRoundtrip {
    fn name(&self) -> &str {
        "cruxx_steps_serialize_roundtrip"
    }
    fn crate_name(&self) -> &str {
        "cruxx"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Unit
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let steps = [
            cruxx::step_pending("t1"),
            cruxx::step_started("t2"),
            cruxx::step_completed("t3", Some("abc"), None),
            cruxx::step_blocked("t4", Some("reason")),
        ];
        for step in &steps {
            let json = serde_json::to_string(step).unwrap();
            let back: cruxx_core::types::step::Step = serde_json::from_str(&json).unwrap();
            if back.name != step.name {
                ctx.fail(&format!("name mismatch after roundtrip: {}", step.name));
            }
            if back.status != step.status {
                ctx.fail(&format!("status mismatch after roundtrip: {}", step.name));
            }
        }
        ctx.result()
    }
}

pub struct CruxxSessionsDirPath;
impl ConformanceTest for CruxxSessionsDirPath {
    fn name(&self) -> &str {
        "cruxx_sessions_dir_path"
    }
    fn crate_name(&self) -> &str {
        "cruxx"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Unit
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let dir = tempfile::TempDir::new().unwrap();
        let path = cruxx::sessions_dir(dir.path());
        let expected = dir.path().join(".ctx").join("sessions");
        if path != expected {
            ctx.fail(&format!("expected {:?}, got {:?}", expected, path));
        }
        ctx.result()
    }
}

pub struct CruxxSessionRoundtrip;
impl ConformanceTest for CruxxSessionRoundtrip {
    fn name(&self) -> &str {
        "cruxx_session_roundtrip"
    }
    fn crate_name(&self) -> &str {
        "cruxx"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Integration
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let dir = tempfile::TempDir::new().unwrap();
        let mut session = Session::start("conformance-agent", dir.path()).unwrap();
        session.record(make_step("s1"));
        session.record(make_step("s2"));
        session.record(make_step("s3"));
        let path = session.finish().unwrap();

        let json = std::fs::read_to_string(&path).unwrap();
        let crux: Crux<TaskGraph> = serde_json::from_str(&json).unwrap();
        ctx.assert_eq(&3usize, &crux.steps.len());
        ctx.assert_eq(&true, &crux.value.is_ok());
        ctx.result()
    }
}

pub struct CruxxSessionFail;
impl ConformanceTest for CruxxSessionFail {
    fn name(&self) -> &str {
        "cruxx_session_fail"
    }
    fn crate_name(&self) -> &str {
        "cruxx"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Integration
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let dir = tempfile::TempDir::new().unwrap();
        let mut session = Session::start("conformance-agent", dir.path()).unwrap();
        session.record(make_step("s1"));
        let err = CruxErr::step_failed("s1", "test failure");
        let path = session.fail(err).unwrap();

        let json = std::fs::read_to_string(&path).unwrap();
        let crux: Crux<TaskGraph> = serde_json::from_str(&json).unwrap();
        ctx.assert_eq(&true, &crux.value.is_err());
        ctx.result()
    }
}

pub fn all() -> Vec<Box<dyn ConformanceTest>> {
    vec![
        Box::new(CruxxStepStartedIsOk),
        Box::new(CruxxStepPendingIsSkipped),
        Box::new(CruxxStepCompletedHasOutput),
        Box::new(CruxxStepBlockedIsErr),
        Box::new(CruxxStepsSerializeRoundtrip),
        Box::new(CruxxSessionsDirPath),
        Box::new(CruxxSessionRoundtrip),
        Box::new(CruxxSessionFail),
    ]
}
