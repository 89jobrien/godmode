use chrono::Utc;
use crux_runtime::types::crux_value::Crux;
use crux_runtime::types::error::CruxErr;
use crux_runtime::types::step::{Step, StepKind, StepStatus};
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
        metadata: Default::default(),
        findings: vec![],
    }
}

#[test]
fn sessions_dir_path_is_under_ctx() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = cruxx::sessions_dir(dir.path());
    assert_eq!(
        path,
        dir.path().join(".ctx").join("godmode").join("sessions")
    );
}

#[test]
fn step_pending_has_skipped_status() {
    let step = cruxx::step_pending("t1");
    assert_eq!(step.name, "t1");
    assert_eq!(step.status, StepStatus::Skipped);
    assert_eq!(step.attempt, 0);
}

#[test]
fn step_started_has_ok_status() {
    let step = cruxx::step_started("t1");
    assert_eq!(step.name, "t1");
    assert_eq!(step.status, StepStatus::Ok);
    assert_eq!(step.attempt, 1);
}

#[test]
fn step_completed_embeds_commit_and_notes_in_output() {
    let step = cruxx::step_completed("t1", Some("deadbeef"), Some("all green"));
    assert_eq!(step.name, "t1");
    assert_eq!(step.status, StepStatus::Ok);
    let output = step.output.unwrap();
    assert_eq!(output["commit"], "deadbeef");
    assert_eq!(output["notes"], "all green");
}

#[test]
fn step_completed_no_output_when_empty() {
    let step = cruxx::step_completed("t1", None, None);
    assert!(step.output.is_none());
}

#[test]
fn step_blocked_has_err_status_and_reason() {
    let step = cruxx::step_blocked("t1", Some("external dep missing"));
    assert_eq!(step.name, "t1");
    assert_eq!(step.status, StepStatus::Err);
    assert_eq!(step.error.as_deref(), Some("external dep missing"));
}

#[test]
fn step_blocked_no_reason_has_no_error() {
    let step = cruxx::step_blocked("t1", None);
    assert!(step.error.is_none());
}

#[test]
fn session_start_finish_writes_file_under_ctx_sessions() {
    let dir = tempfile::TempDir::new().unwrap();
    let session = Session::start("test-agent", dir.path()).unwrap();
    let path = session.finish().unwrap();

    assert!(path.exists());
    assert!(path.starts_with(dir.path().join(".ctx").join("godmode").join("sessions")));
}

#[test]
fn session_finish_deserialises_to_crux_task_graph_with_correct_step_count() {
    let dir = tempfile::TempDir::new().unwrap();
    let mut session = Session::start("test-agent", dir.path()).unwrap();
    session.record(make_step("s1"));
    session.record(make_step("s2"));
    session.record(make_step("s3"));
    let path = session.finish().unwrap();

    let json = std::fs::read_to_string(&path).unwrap();
    let crux: Crux<TaskGraph> = serde_json::from_str(&json).unwrap();
    assert_eq!(crux.steps.len(), 3);
}

#[test]
fn session_fail_writes_err_value() {
    let dir = tempfile::TempDir::new().unwrap();
    let mut session = Session::start("test-agent", dir.path()).unwrap();
    session.record(make_step("s1"));
    let err = CruxErr::step_failed("s1", "something broke");
    let path = session.fail(err).unwrap();

    let json = std::fs::read_to_string(&path).unwrap();
    let crux: Crux<TaskGraph> = serde_json::from_str(&json).unwrap();
    assert!(crux.value.is_err());
}

#[test]
fn steps_serialize_to_valid_json() {
    // All step constructors produce values that round-trip through serde_json.
    let steps = [
        cruxx::step_pending("t1"),
        cruxx::step_started("t2"),
        cruxx::step_completed("t3", Some("abc"), None),
        cruxx::step_blocked("t4", Some("reason")),
    ];
    for step in &steps {
        let json = serde_json::to_string(step).unwrap();
        let back: crux_runtime::types::step::Step = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, step.name);
        assert_eq!(back.status, step.status);
    }
}
