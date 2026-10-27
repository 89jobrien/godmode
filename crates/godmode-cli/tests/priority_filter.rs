//! Tests priority filtering for `task list` and `task next` through the CLI.

/// Integration tests for --priority filter on `task list` and `task next`.
///
/// These tests build a temporary task graph on disk and invoke the compiled
/// `godmode` binary, asserting stdout content and exit codes.
mod common;

/// Create a temp directory containing a pre-populated `.ctx/godmode/tasks.yaml`.
fn setup_graph(yaml: &str) -> tempfile::TempDir {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let ctx = dir.path().join(".ctx").join("godmode");
    std::fs::create_dir_all(&ctx).unwrap();
    std::fs::write(ctx.join("tasks.yaml"), yaml).unwrap();
    dir
}

const MIXED_GRAPH: &str = r#"tasks:
  - id: t1
    title: High priority task
    status: pending
    depends_on: []
    priority: high
  - id: t2
    title: Normal priority task
    status: pending
    depends_on: []
  - id: t3
    title: Low priority task
    status: pending
    depends_on: []
    priority: low
"#;

// ── task list --priority high ──────────────────────────────────────────────

#[test]
fn task_list_priority_high_shows_only_high() {
    let dir = setup_graph(MIXED_GRAPH);
    let out = common::run_godmode(dir.path(), &["task", "list", "--priority", "high"]);

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "expected exit 0, got {:?}\nstdout: {stdout}\nstderr: {}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        stdout.contains("High priority task"),
        "expected 'High priority task' in output:\n{stdout}"
    );
    assert!(
        !stdout.contains("Normal priority task"),
        "expected normal task absent:\n{stdout}"
    );
    assert!(
        !stdout.contains("Low priority task"),
        "expected low task absent:\n{stdout}"
    );
}

#[test]
fn task_list_priority_low_shows_only_low() {
    let dir = setup_graph(MIXED_GRAPH);
    let out = common::run_godmode(dir.path(), &["task", "list", "--priority", "low"]);

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "expected exit 0, got {:?}\nstdout: {stdout}\nstderr: {}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(stdout.contains("Low priority task"), "stdout:\n{stdout}");
    assert!(!stdout.contains("High priority task"), "stdout:\n{stdout}");
}

#[test]
fn task_list_no_priority_shows_all() {
    let dir = setup_graph(MIXED_GRAPH);
    let out = common::run_godmode(dir.path(), &["task", "list"]);

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "stdout: {stdout}");
    assert!(stdout.contains("High priority task"), "stdout:\n{stdout}");
    assert!(stdout.contains("Normal priority task"), "stdout:\n{stdout}");
    assert!(stdout.contains("Low priority task"), "stdout:\n{stdout}");
}

// ── task next --priority high ──────────────────────────────────────────────

#[test]
fn task_next_priority_high_shows_only_high_runnable() {
    let dir = setup_graph(MIXED_GRAPH);
    let out = common::run_godmode(dir.path(), &["task", "next", "--priority", "high"]);

    let stdout = String::from_utf8_lossy(&out.stdout);
    // Exit 0 when at least one result; exit 1 when empty — high task is pending+runnable.
    assert!(
        out.status.success(),
        "expected exit 0, got {:?}\nstdout: {stdout}\nstderr: {}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(stdout.contains("High priority task"), "stdout:\n{stdout}");
    assert!(
        !stdout.contains("Normal priority task"),
        "stdout:\n{stdout}"
    );
}

#[test]
fn task_next_priority_high_empty_exits_nonzero() {
    // Graph with only a normal-priority task — `task next --priority high` should exit 1.
    let yaml = r#"tasks:
  - id: t1
    title: Normal only
    status: pending
    depends_on: []
"#;
    let dir = setup_graph(yaml);
    let out = common::run_godmode(dir.path(), &["task", "next", "--priority", "high"]);

    assert!(
        !out.status.success(),
        "expected non-zero exit when no high tasks, got 0\nstdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}
