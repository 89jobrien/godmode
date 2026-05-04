/// Integration tests for --priority filter on `task list` and `task next`.
///
/// These tests build a temporary task graph on disk and invoke the compiled
/// `godmode` binary, asserting stdout content and exit codes.
use std::process::Command;

/// Path to the compiled godmode binary produced by `cargo test --package godmode-cli`.
fn godmode_bin() -> std::path::PathBuf {
    // cargo nextest / cargo test sets CARGO_BIN_EXE_godmode automatically.
    if let Ok(p) = std::env::var("CARGO_BIN_EXE_godmode") {
        return std::path::PathBuf::from(p);
    }
    // Fallback: walk up from CARGO_MANIFEST_DIR to workspace root then target/debug.
    let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest.parent().unwrap().parent().unwrap();
    workspace_root.join("target/debug/godmode")
}

/// Create a temp directory containing a pre-populated `.ctx/GODMODE.tasks.yaml`.
fn setup_graph(yaml: &str) -> tempfile::TempDir {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let ctx = dir.path().join(".ctx");
    std::fs::create_dir_all(&ctx).unwrap();
    std::fs::write(ctx.join("GODMODE.tasks.yaml"), yaml).unwrap();
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
    let out = Command::new(godmode_bin())
        .args(["task", "list", "--priority", "high"])
        .current_dir(dir.path())
        .output()
        .expect("run godmode");

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
    let out = Command::new(godmode_bin())
        .args(["task", "list", "--priority", "low"])
        .current_dir(dir.path())
        .output()
        .expect("run godmode");

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
    let out = Command::new(godmode_bin())
        .args(["task", "list"])
        .current_dir(dir.path())
        .output()
        .expect("run godmode");

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
    let out = Command::new(godmode_bin())
        .args(["task", "next", "--priority", "high"])
        .current_dir(dir.path())
        .output()
        .expect("run godmode");

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
    let out = Command::new(godmode_bin())
        .args(["task", "next", "--priority", "high"])
        .current_dir(dir.path())
        .output()
        .expect("run godmode");

    assert!(
        !out.status.success(),
        "expected non-zero exit when no high tasks, got 0\nstdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}
