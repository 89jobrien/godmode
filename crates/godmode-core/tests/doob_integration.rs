use godmode_core::integrations::doob;

const DOOB_LIST_JSON: &[u8] = br#"{
  "count": 2,
  "todos": [
    {"id": "a", "content": "First task", "status": "completed", "priority": 4},
    {"id": "b", "content": "Second task", "status": "pending", "priority": 3}
  ]
}"#;

#[test]
fn parse_todo_list_parses_json() {
    let v = doob::parse_todo_list(DOOB_LIST_JSON).unwrap();
    assert_eq!(v["count"], 2);
    assert_eq!(v["todos"][0]["content"], "First task");
}

#[test]
fn parse_todo_list_errors_on_invalid_json() {
    let result = doob::parse_todo_list(b"not json");
    assert!(result.is_err());
}

#[test]
fn find_next_pending_returns_first_pending() {
    let v = doob::parse_todo_list(DOOB_LIST_JSON).unwrap();
    let next = doob::find_next_pending(&v).unwrap();
    assert_eq!(next["content"], "Second task");
    assert_eq!(next["status"], "pending");
}

#[test]
fn find_next_pending_returns_none_when_all_done() {
    let v: serde_json::Value = serde_json::json!({
        "count": 1,
        "todos": [{"id": "a", "content": "Done", "status": "completed"}]
    });
    assert!(doob::find_next_pending(&v).is_none());
}

#[test]
fn find_next_pending_returns_none_on_empty_list() {
    let v: serde_json::Value = serde_json::json!({ "count": 0, "todos": [] });
    assert!(doob::find_next_pending(&v).is_none());
}
