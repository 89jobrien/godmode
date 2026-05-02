mod fake_bin;
use fake_bin::FakeBin;
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

// ── D: doob write ────────────────────────────────────────────────────────────

#[test]
fn todo_done_calls_doob_with_correct_args() {
    let fake = FakeBin::new("doob").echo_argv().build();
    let argv = doob::todo_done_args("uuid-abc");
    assert_eq!(argv, vec!["todo", "complete", "uuid-abc"]);
    let _ = fake; // keep alive
}

#[test]
fn todo_add_args_includes_project_and_title() {
    let argv = doob::todo_add_args("godmode", "Write failing test for FooAdapter");
    assert_eq!(
        argv,
        vec![
            "todo",
            "add",
            "-p",
            "godmode",
            "Write failing test for FooAdapter"
        ]
    );
}

#[test]
fn import_todos_converts_pending_todos_to_tasks() {
    let raw = br#"{
      "count": 2,
      "todos": [
        {"id": "uuid-1", "content": "First task", "status": "pending", "priority": 3},
        {"id": "uuid-2", "content": "Second task", "status": "completed", "priority": 2}
      ]
    }"#;
    let v = doob::parse_todo_list(raw).unwrap();
    let tasks = doob::todos_to_tasks(&v);
    // only pending todos become tasks
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].title, "First task");
    assert_eq!(tasks[0].notes, "doob:uuid-1");
}

#[test]
fn import_todos_skips_completed() {
    let raw = br#"{"count":1,"todos":[{"id":"x","content":"Done","status":"completed"}]}"#;
    let v = doob::parse_todo_list(raw).unwrap();
    let tasks = doob::todos_to_tasks(&v);
    assert!(tasks.is_empty());
}
