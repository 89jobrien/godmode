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
    assert!(line.get("ts").is_some());
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
