//! Integration test: full session lifecycle (handon -> start -> complete -> handoff).
//!
//! Exercises the composed flow through `Session`, verifying that state persists
//! to disk across transitions and the summary reflects the full journey.

use godmode_core::config::Config;
use godmode_core::graph;
use godmode_core::model::{Status, Task};
use godmode_core::session::Session;

fn disabled_config() -> Config {
    let mut cfg = Config::default();
    cfg.integrations.rx = false;
    cfg.integrations.cruxx = false;
    cfg.integrations.doob = false;
    cfg.integrations.hj = false;
    cfg
}

#[test]
fn full_lifecycle_handon_start_complete_handoff() {
    let dir = tempfile::TempDir::new().unwrap();
    let cfg = disabled_config();

    // 1. Open session, add tasks with a dependency chain: t1 -> t2 -> t3
    let mut s = Session::open_with_config(dir.path(), &cfg).unwrap();
    s.add_task(Task::new("t1", "Write tests")).unwrap();
    let mut t2 = Task::new("t2", "Implement feature");
    t2.depends_on = vec!["t1".into()];
    s.add_task(t2).unwrap();
    let mut t3 = Task::new("t3", "Refactor");
    t3.depends_on = vec!["t2".into()];
    s.add_task(t3).unwrap();
    s.save().unwrap();

    // 2. Verify initial state: 3 pending, only t1 runnable
    let runnable = graph::runnable(s.graph());
    assert_eq!(runnable.len(), 1);
    assert_eq!(runnable[0].id, "t1");

    // 3. Start and complete t1
    s.start_task("t1").unwrap();
    assert_eq!(
        s.graph()
            .tasks
            .iter()
            .find(|t| t.id == "t1")
            .unwrap()
            .status,
        Status::Running
    );
    s.complete_task("t1", Some("abc123"), Some("tests pass"))
        .unwrap();

    // 4. Reload from disk — verify t1 persisted as Done
    let reloaded = graph::load(dir.path()).unwrap();
    let t1 = reloaded.tasks.iter().find(|t| t.id == "t1").unwrap();
    assert_eq!(t1.status, Status::Done);
    assert_eq!(t1.commit.as_deref(), Some("abc123"));
    assert!(t1.completed.is_some());

    // 5. t2 should now be runnable (dep t1 is done)
    let runnable2 = graph::runnable(&reloaded);
    assert_eq!(runnable2.len(), 1);
    assert_eq!(runnable2[0].id, "t2");

    // 6. Start t2, then block it
    s.start_task("t2").unwrap();
    s.block_task("t2", "waiting on code review").unwrap();
    let t2_state = s.graph().tasks.iter().find(|t| t.id == "t2").unwrap();
    assert_eq!(t2_state.status, Status::Blocked);
    assert_eq!(t2_state.notes, "waiting on code review");

    // 7. Unblock and complete t2
    s.unblock_task("t2").unwrap();
    assert_eq!(
        s.graph()
            .tasks
            .iter()
            .find(|t| t.id == "t2")
            .unwrap()
            .status,
        Status::Pending
    );
    s.start_task("t2").unwrap();
    s.complete_task("t2", Some("def456"), None).unwrap();

    // 8. Complete t3
    s.start_task("t3").unwrap();
    s.complete_task("t3", None, None).unwrap();

    // 9. Final summary: all done
    let summary = s.summary();
    assert_eq!(summary.done, 3);
    assert_eq!(summary.running, 0);
    assert_eq!(summary.pending, 0);
    assert_eq!(summary.blocked, 0);
    assert_eq!(summary.tasks.len(), 3);
    assert!(
        summary.total_duration_ms < 5000,
        "total duration should be small in test"
    );

    // 10. Verify final state persisted to disk
    let final_graph = graph::load(dir.path()).unwrap();
    assert!(final_graph.tasks.iter().all(|t| t.status == Status::Done));
}

#[test]
fn lifecycle_invalid_transitions_are_rejected() {
    let dir = tempfile::TempDir::new().unwrap();
    let cfg = disabled_config();
    let mut s = Session::open_with_config(dir.path(), &cfg).unwrap();

    let mut t1 = Task::new("t1", "First");
    let mut t2 = Task::new("t2", "Second");
    t2.depends_on = vec!["t1".into()];
    s.add_task(t1).unwrap();
    s.add_task(t2).unwrap();

    // Cannot complete a pending task (must be running first)
    assert!(s.complete_task("t1", None, None).is_err());

    // Cannot start t2 (dep t1 not done)
    assert!(s.start_task("t2").is_err());

    // Cannot unblock a non-blocked task
    assert!(s.unblock_task("t1").is_err());

    // Start t1, cannot start again
    s.start_task("t1").unwrap();
    assert!(s.start_task("t1").is_err());

    // Complete t1, cannot complete again
    s.complete_task("t1", None, None).unwrap();
    assert!(s.complete_task("t1", None, None).is_err());
}

#[test]
fn lifecycle_clear_done_preserves_active_work() {
    let dir = tempfile::TempDir::new().unwrap();
    let cfg = disabled_config();
    let mut s = Session::open_with_config(dir.path(), &cfg).unwrap();

    s.add_task(Task::new("t1", "Done soon")).unwrap();
    s.add_task(Task::new("t2", "Still pending")).unwrap();
    s.add_task(Task::new("t3", "Will be running")).unwrap();

    s.start_task("t1").unwrap();
    s.complete_task("t1", None, None).unwrap();
    s.start_task("t3").unwrap();

    let cleared = s.clear_tasks(true); // done_only
    assert_eq!(cleared, 1);
    assert_eq!(s.graph().tasks.len(), 2);

    let ids: Vec<&str> = s.graph().tasks.iter().map(|t| t.id.as_str()).collect();
    assert!(ids.contains(&"t2"));
    assert!(ids.contains(&"t3"));

    // Verify persisted
    let reloaded = graph::load(dir.path()).unwrap();
    assert_eq!(reloaded.tasks.len(), 2);
}
