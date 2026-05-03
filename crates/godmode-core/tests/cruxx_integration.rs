use godmode_core::integrations::cruxx;

use tempfile::TempDir;

#[test]
fn trace_file_path_is_under_ctx() {
    let dir = TempDir::new().unwrap();
    let path = cruxx::trace_file(dir.path());
    assert_eq!(path, dir.path().join(".ctx").join("GODMODE.trace.jsonl"));
}

#[test]
fn append_event_writes_valid_jsonl_line() {
    let dir = TempDir::new().unwrap();
    let mut event = cruxx::TaskEvent::started("t1", "Write failing test");
    event.crate_name = Some("godmode-core".into());
    cruxx::append_event(dir.path(), &event).unwrap();

    let content = std::fs::read_to_string(cruxx::trace_file(dir.path())).unwrap();
    let line: serde_json::Value = serde_json::from_str(content.trim()).unwrap();
    assert_eq!(line["state"], "running");
    assert_eq!(line["step_name"], "t1");
    assert_eq!(line["title"], "Write failing test");
    assert_eq!(line["crate_name"], "godmode-core");
    // ts is now part of the struct — always present
    let ts = line.get("ts").expect("ts field must be present");
    assert!(ts.as_str().unwrap().contains('T'), "ts should be RFC 3339");
}

#[test]
fn ts_is_present_in_all_event_types() {
    // ts is embedded in the struct, so direct serialisation (without append_event) includes it.
    let events = [
        cruxx::TaskEvent::pending("t1", "A"),
        cruxx::TaskEvent::started("t1", "A"),
        cruxx::TaskEvent::completed("t1", "A", None, None),
        cruxx::TaskEvent::blocked("t1", "A", None),
    ];
    for event in &events {
        let json = serde_json::to_value(event).unwrap();
        assert!(
            json.get("ts").is_some(),
            "ts missing for state {:?}",
            event.state
        );
    }
}

#[test]
fn append_event_appends_multiple_lines() {
    let dir = TempDir::new().unwrap();
    for id in ["t1", "t2"] {
        let event = cruxx::TaskEvent::completed(id, "task", Some("abc1234".into()), None);
        cruxx::append_event(dir.path(), &event).unwrap();
    }
    let content = std::fs::read_to_string(cruxx::trace_file(dir.path())).unwrap();
    assert_eq!(content.lines().count(), 2);
}

#[test]
fn completed_event_includes_commit() {
    let dir = TempDir::new().unwrap();
    let event = cruxx::TaskEvent::completed(
        "t1",
        "Impl",
        Some("deadbeef".into()),
        Some("all green".into()),
    );
    cruxx::append_event(dir.path(), &event).unwrap();
    let content = std::fs::read_to_string(cruxx::trace_file(dir.path())).unwrap();
    let line: serde_json::Value = serde_json::from_str(content.trim()).unwrap();
    assert_eq!(line["state"], "completed");
    assert_eq!(line["commit"], "deadbeef");
    assert_eq!(line["notes"], "all green");
}

#[test]
fn pending_event_emits_pending_state() {
    let dir = TempDir::new().unwrap();
    let event = cruxx::TaskEvent::pending("t1", "New task");
    cruxx::append_event(dir.path(), &event).unwrap();
    let content = std::fs::read_to_string(cruxx::trace_file(dir.path())).unwrap();
    let line: serde_json::Value = serde_json::from_str(content.trim()).unwrap();
    assert_eq!(line["state"], "pending");
    assert_eq!(line["step_name"], "t1");
}

#[test]
fn blocked_event_emits_cancelled_not_failed() {
    let dir = TempDir::new().unwrap();
    let event = cruxx::TaskEvent::blocked("t1", "Stuck", Some("external dep missing".into()));
    cruxx::append_event(dir.path(), &event).unwrap();
    let content = std::fs::read_to_string(cruxx::trace_file(dir.path())).unwrap();
    let line: serde_json::Value = serde_json::from_str(content.trim()).unwrap();
    // Must be "cancelled" — not "failed" — a blocked task was externally stopped, not internally errored.
    assert_eq!(line["state"], "cancelled");
    assert_eq!(line["reason"], "external dep missing");
}

#[test]
fn add_traced_emits_pending_event() {
    use godmode_core::graph;
    use godmode_core::model::{Task, TaskGraph};

    let dir = TempDir::new().unwrap();
    let mut g = TaskGraph::default();
    graph::add_traced(&mut g, Task::new("t1", "My task"), Some(dir.path())).unwrap();

    let content = std::fs::read_to_string(cruxx::trace_file(dir.path())).unwrap();
    let line: serde_json::Value = serde_json::from_str(content.trim()).unwrap();
    assert_eq!(line["state"], "pending");
    assert_eq!(line["step_name"], "t1");
    assert_eq!(line["title"], "My task");
}
