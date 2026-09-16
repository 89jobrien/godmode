mod fake_bin;
use fake_bin::FakeBin;
use godmode_core::integrations::doob;
use godmode_core::model::Task;

#[derive(Default)]
struct FakePublisher {
    published: Vec<(String, String)>,
}

impl doob::TodoPublisher for FakePublisher {
    fn publish(&mut self, project: &str, title: &str) -> anyhow::Result<String> {
        self.published.push((project.into(), title.into()));
        Ok(format!("doob-{}", self.published.len()))
    }
}

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
            "Write failing test for FooAdapter",
            "--json"
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
    assert_eq!(tasks[0].doob_id(), Some("uuid-1"));
}

#[test]
fn import_todos_skips_completed() {
    let raw = br#"{"count":1,"todos":[{"id":"x","content":"Done","status":"completed"}]}"#;
    let v = doob::parse_todo_list(raw).unwrap();
    let tasks = doob::todos_to_tasks(&v);
    assert!(tasks.is_empty());
}

#[test]
fn publishes_local_tasks_and_records_returned_identifier() {
    let mut tasks = vec![Task::new("t1", "Local task")];
    let mut publisher = FakePublisher::default();

    let published = doob::publish_tasks(&mut publisher, "godmode", &mut tasks).unwrap();

    assert_eq!(published, 1);
    assert_eq!(
        publisher.published,
        vec![("godmode".into(), "Local task".into())]
    );
    assert_eq!(tasks[0].doob_id(), Some("doob-1"));
}

#[test]
fn publishing_is_idempotent_for_tasks_with_doob_provenance() {
    let mut task = Task::new("t1", "Already published");
    task.set_doob_id("existing-id");
    let mut tasks = vec![task];
    let mut publisher = FakePublisher::default();

    let published = doob::publish_tasks(&mut publisher, "godmode", &mut tasks).unwrap();

    assert_eq!(published, 0);
    assert!(publisher.published.is_empty());
    assert_eq!(tasks[0].doob_id(), Some("existing-id"));
}

#[test]
fn returned_identifier_roundtrips_as_structured_provenance() {
    let mut task = Task::new("t1", "Published task");
    task.set_doob_id("uuid-123");

    let yaml = serde_yaml::to_string(&task).unwrap();
    let restored: Task = serde_yaml::from_str(&yaml).unwrap();

    assert!(yaml.contains("provenance:"), "yaml: {yaml}");
    assert!(yaml.contains("doob:"), "yaml: {yaml}");
    assert_eq!(restored.doob_id(), Some("uuid-123"));
}

#[test]
fn imported_legacy_notes_remain_compatible_with_completion_sync() {
    let raw =
        br#"{"count":1,"todos":[{"id":"legacy-id","content":"Imported","status":"pending"}]}"#;
    let value = doob::parse_todo_list(raw).unwrap();
    let tasks = doob::todos_to_tasks(&value);

    assert_eq!(tasks[0].notes, "doob:legacy-id");
    assert_eq!(doob::todo_id(&tasks[0]), Some("legacy-id"));

    let mut legacy = Task::new("t1", "Legacy imported task");
    legacy.notes = "doob:legacy-id".into();
    let mut legacy_tasks = vec![legacy];
    let mut publisher = FakePublisher::default();
    assert_eq!(
        doob::publish_tasks(&mut publisher, "godmode", &mut legacy_tasks).unwrap(),
        0
    );
    assert!(publisher.published.is_empty());
    assert_eq!(legacy_tasks[0].doob_id(), Some("legacy-id"));
}
