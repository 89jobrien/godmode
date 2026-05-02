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
    let event = cruxx::TaskEvent {
        kind: cruxx::EventKind::Started,
        task_id: "t1".into(),
        title: "Write failing test".into(),
        crate_name: Some("godmode-core".into()),
        commit: None,
        notes: None,
    };
    cruxx::append_event(dir.path(), &event).unwrap();

    let content = std::fs::read_to_string(cruxx::trace_file(dir.path())).unwrap();
    let line: serde_json::Value = serde_json::from_str(content.trim()).unwrap();
    assert_eq!(line["kind"], "started");
    assert_eq!(line["task_id"], "t1");
    assert_eq!(line["title"], "Write failing test");
    assert_eq!(line["crate_name"], "godmode-core");
    assert!(line.get("ts").is_some());
}

#[test]
fn append_event_appends_multiple_lines() {
    let dir = TempDir::new().unwrap();
    for id in ["t1", "t2"] {
        let event = cruxx::TaskEvent {
            kind: cruxx::EventKind::Completed,
            task_id: id.into(),
            title: "task".into(),
            crate_name: None,
            commit: Some("abc1234".into()),
            notes: None,
        };
        cruxx::append_event(dir.path(), &event).unwrap();
    }
    let content = std::fs::read_to_string(cruxx::trace_file(dir.path())).unwrap();
    assert_eq!(content.lines().count(), 2);
}

#[test]
fn completed_event_includes_commit() {
    let dir = TempDir::new().unwrap();
    let event = cruxx::TaskEvent {
        kind: cruxx::EventKind::Completed,
        task_id: "t1".into(),
        title: "Impl".into(),
        crate_name: None,
        commit: Some("deadbeef".into()),
        notes: Some("all green".into()),
    };
    cruxx::append_event(dir.path(), &event).unwrap();
    let content = std::fs::read_to_string(cruxx::trace_file(dir.path())).unwrap();
    let line: serde_json::Value = serde_json::from_str(content.trim()).unwrap();
    assert_eq!(line["commit"], "deadbeef");
    assert_eq!(line["notes"], "all green");
}
