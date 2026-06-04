//! Conformance tests for godmode_core::integrations::crux — Step constructor API.

use chrono::Utc;
use crux_runtime::types::crux_value::Crux;
use crux_runtime::types::error::CruxErr;
use crux_runtime::types::step::{Step, StepKind, StepStatus};
use godmode_core::integrations::crux;
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
        metadata: Default::default(),
        findings: vec![],
    }
}

use crate::harness::{ConformanceTest, TestCategory, TestContext, TestResult};

pub struct CruxStepStartedIsOk;
impl ConformanceTest for CruxStepStartedIsOk {
    fn name(&self) -> &str {
        "crux_step_started_is_ok"
    }
    fn crate_name(&self) -> &str {
        "crux"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Unit
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let step = crux::step_started("t1");
        ctx.assert_str_eq("t1", &step.name);
        ctx.assert_eq(&StepStatus::Ok, &step.status);
        ctx.assert_eq(&1u32, &step.attempt);
        ctx.result()
    }
}

pub struct CruxStepPendingIsSkipped;
impl ConformanceTest for CruxStepPendingIsSkipped {
    fn name(&self) -> &str {
        "crux_step_pending_is_skipped"
    }
    fn crate_name(&self) -> &str {
        "crux"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Unit
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let step = crux::step_pending("t0");
        ctx.assert_str_eq("t0", &step.name);
        ctx.assert_eq(&StepStatus::Skipped, &step.status);
        ctx.assert_eq(&0u32, &step.attempt);
        ctx.result()
    }
}

pub struct CruxStepCompletedHasOutput;
impl ConformanceTest for CruxStepCompletedHasOutput {
    fn name(&self) -> &str {
        "crux_step_completed_has_output"
    }
    fn crate_name(&self) -> &str {
        "crux"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Unit
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let step = crux::step_completed("t2", Some("deadbeef"), Some("all green"));
        ctx.assert_eq(&StepStatus::Ok, &step.status);
        let output = step.output.as_ref().expect("output should be Some");
        ctx.assert_str_eq("deadbeef", output["commit"].as_str().unwrap_or(""));
        ctx.assert_str_eq("all green", output["notes"].as_str().unwrap_or(""));
        ctx.result()
    }
}

pub struct CruxStepBlockedIsErr;
impl ConformanceTest for CruxStepBlockedIsErr {
    fn name(&self) -> &str {
        "crux_step_blocked_is_err"
    }
    fn crate_name(&self) -> &str {
        "crux"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Unit
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let step = crux::step_blocked("t3", Some("external dep missing"));
        ctx.assert_eq(&StepStatus::Err, &step.status);
        ctx.assert_str_eq("external dep missing", step.error.as_deref().unwrap_or(""));
        ctx.result()
    }
}

pub struct CruxStepsSerializeRoundtrip;
impl ConformanceTest for CruxStepsSerializeRoundtrip {
    fn name(&self) -> &str {
        "crux_steps_serialize_roundtrip"
    }
    fn crate_name(&self) -> &str {
        "crux"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Unit
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let steps = [
            crux::step_pending("t1"),
            crux::step_started("t2"),
            crux::step_completed("t3", Some("abc"), None),
            crux::step_blocked("t4", Some("reason")),
        ];
        for step in &steps {
            let json = serde_json::to_string(step).unwrap();
            let back: crux_runtime::types::step::Step = serde_json::from_str(&json).unwrap();
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

pub struct CruxSessionsDirPath;
impl ConformanceTest for CruxSessionsDirPath {
    fn name(&self) -> &str {
        "crux_sessions_dir_path"
    }
    fn crate_name(&self) -> &str {
        "crux"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Unit
    }
    fn run(&self, ctx: &mut TestContext) -> TestResult {
        let dir = tempfile::TempDir::new().unwrap();
        let path = crux::sessions_dir(dir.path());
        let expected = dir.path().join(".ctx").join("sessions");
        if path != expected {
            ctx.fail(&format!("expected {:?}, got {:?}", expected, path));
        }
        ctx.result()
    }
}

pub struct CruxSessionRoundtrip;
impl ConformanceTest for CruxSessionRoundtrip {
    fn name(&self) -> &str {
        "crux_session_roundtrip"
    }
    fn crate_name(&self) -> &str {
        "crux"
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

pub struct CruxSessionFail;
impl ConformanceTest for CruxSessionFail {
    fn name(&self) -> &str {
        "crux_session_fail"
    }
    fn crate_name(&self) -> &str {
        "crux"
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
        Box::new(CruxStepStartedIsOk),
        Box::new(CruxStepPendingIsSkipped),
        Box::new(CruxStepCompletedHasOutput),
        Box::new(CruxStepBlockedIsErr),
        Box::new(CruxStepsSerializeRoundtrip),
        Box::new(CruxSessionsDirPath),
        Box::new(CruxSessionRoundtrip),
        Box::new(CruxSessionFail),
    ]
}
