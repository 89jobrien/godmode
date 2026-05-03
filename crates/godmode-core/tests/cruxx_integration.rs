use cruxx_core::types::step::StepStatus;
use godmode_core::integrations::cruxx;

#[test]
fn sessions_dir_path_is_under_ctx() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = cruxx::sessions_dir(dir.path());
    assert_eq!(path, dir.path().join(".ctx").join("sessions"));
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
        let back: cruxx_core::types::step::Step = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, step.name);
        assert_eq!(back.status, step.status);
    }
}
