mod fake_bin;
use fake_bin::FakeBin;
use godmode_core::integrations::gh;

const ISSUE_LIST_JSON: &str = r#"[
  {"number": 1, "title": "Fix the bug", "body": "Detailed description", "labels": []},
  {"number": 2, "title": "Add a feature", "body": "", "labels": [{"name": "epic"}]},
  {"number": 3, "title": "Docs update", "body": null, "labels": [{"name": "docs"}]}
]"#;

// ── unit: parse_issue_list ────────────────────────────────────────────────────

#[test]
fn parse_issue_list_returns_array() {
    let v = gh::parse_issue_list(ISSUE_LIST_JSON.as_bytes()).unwrap();
    assert!(v.is_array());
    assert_eq!(v.as_array().unwrap().len(), 3);
}

// ── unit: issues_to_tasks ─────────────────────────────────────────────────────

#[test]
fn issues_to_tasks_maps_all_without_label_filter() {
    let v = gh::parse_issue_list(ISSUE_LIST_JSON.as_bytes()).unwrap();
    let tasks = gh::issues_to_tasks(&v, None);
    assert_eq!(tasks.len(), 3);
    assert_eq!(tasks[0].id, "gh-1");
    assert_eq!(tasks[0].title, "Fix the bug");
    assert_eq!(tasks[0].notes, "Detailed description");
    assert_eq!(tasks[1].id, "gh-2");
    assert!(
        tasks[1].notes.is_empty(),
        "empty body should leave notes empty"
    );
}

#[test]
fn issues_to_tasks_filters_by_label() {
    let v = gh::parse_issue_list(ISSUE_LIST_JSON.as_bytes()).unwrap();
    let tasks = gh::issues_to_tasks(&v, Some("epic"));
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].id, "gh-2");
}

#[test]
fn issues_to_tasks_label_no_match_returns_empty() {
    let v = gh::parse_issue_list(ISSUE_LIST_JSON.as_bytes()).unwrap();
    let tasks = gh::issues_to_tasks(&v, Some("nonexistent"));
    assert!(tasks.is_empty());
}

// ── integration: pull_issues via FakeBin ─────────────────────────────────────

#[test]
fn pull_issues_returns_tasks_from_gh_output() {
    let fake = FakeBin::new("gh").stdout(ISSUE_LIST_JSON).build();
    // SAFETY: test-only PATH manipulation
    unsafe {
        std::env::set_var("PATH", fake.path_with());
    }
    let tasks = gh::pull_issues(None, None).unwrap();
    assert_eq!(tasks.len(), 3);
    assert_eq!(tasks[0].id, "gh-1");
    let _ = fake;
}

#[test]
fn pull_issues_idempotent_no_duplicates() {
    use godmode_core::{graph, model::TaskGraph};
    let v = gh::parse_issue_list(ISSUE_LIST_JSON.as_bytes()).unwrap();
    let tasks1 = gh::issues_to_tasks(&v, None);
    let tasks2 = gh::issues_to_tasks(&v, None);

    let mut g = TaskGraph::default();
    for task in tasks1 {
        let _ = graph::add(&mut g, task);
    }
    // Second pull: skip tasks that already exist
    for task in tasks2 {
        match graph::add(&mut g, task) {
            Ok(()) => {}
            Err(e) if e.to_string().contains("already exists") => {}
            Err(e) => panic!("unexpected error: {e}"),
        }
    }
    assert_eq!(g.tasks.len(), 3, "no duplicates after two pulls");
}
