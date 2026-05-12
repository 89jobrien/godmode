//! Conformance tests: verify `godmode` binary exit-code and JSON-output contracts.

use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

use tempfile::TempDir;

/// Path to the compiled `godmode` binary.
///
/// Walks from CARGO_MANIFEST_DIR up to the workspace root, then checks
/// `target/debug/godmode` and `target/release/godmode`.
fn godmode_bin() -> PathBuf {
    // CARGO_MANIFEST_DIR is crates/godmode-core; workspace root is two levels up.
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root");

    // Prefer the profile that matches current build (debug first, then release).
    for profile in &["debug", "release"] {
        let candidate = workspace_root.join("target").join(profile).join("godmode");
        if candidate.exists() {
            return candidate;
        }
    }

    panic!(
        "godmode binary not found under {}/target/{{debug,release}}. \
         Run `cargo build -p godmode-cli` first.",
        workspace_root.display()
    )
}

/// Run `godmode` in `dir` with the given args. Returns `(exit_code, stdout, stderr)`.
fn run(dir: &std::path::Path, args: &[&str]) -> (i32, String, String) {
    let out = Command::new(godmode_bin())
        .args(args)
        .current_dir(dir)
        .output()
        .expect("failed to run godmode");
    let code = out.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    (code, stdout, stderr)
}

// ── empty_graph_task_next_exits_2 ────────────────────────────────────────────

#[test]
fn empty_graph_task_next_exits_2() {
    let dir = TempDir::new().unwrap();
    let (code, _stdout, _stderr) = run(dir.path(), &["task", "next", "--json"]);
    assert_eq!(
        code, 2,
        "task next on empty graph should exit 2 (empty result)"
    );
}

// ── empty_graph_task_list_exits_0 ────────────────────────────────────────────

#[test]
fn empty_graph_task_list_exits_0() {
    let dir = TempDir::new().unwrap();
    let (code, stdout, _stderr) = run(dir.path(), &["task", "list", "--json"]);
    assert_eq!(code, 0, "task list on empty graph should exit 0");
    // stdout must be a parseable empty JSON array.
    let parsed: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("stdout should be valid JSON");
    assert!(
        parsed.is_array(),
        "task list --json output should be a JSON array, got: {stdout}"
    );
    assert_eq!(parsed.as_array().unwrap().len(), 0);
}

// ── handon_exits_0 ───────────────────────────────────────────────────────────

#[test]
fn handon_exits_0() {
    let dir = TempDir::new().unwrap();
    let (code, _stdout, _stderr) = run(dir.path(), &["handon"]);
    assert_eq!(code, 0, "handon should exit 0");
}

// ── handoff_no_running_exits_0 ───────────────────────────────────────────────

#[test]
fn handoff_no_running_exits_0() {
    let dir = TempDir::new().unwrap();
    let (code, _stdout, _stderr) = run(dir.path(), &["handoff"]);
    assert_eq!(code, 0, "handoff with no running tasks should exit 0");
}

// ── plan_ingest_produces_tasks ───────────────────────────────────────────────

#[test]
fn plan_ingest_produces_tasks() {
    let dir = TempDir::new().unwrap();
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/simple-plan.md");

    let (code, _stdout, _stderr) = run(dir.path(), &["plan", "ingest", fixture.to_str().unwrap()]);
    assert_eq!(code, 0, "plan ingest should exit 0");

    // Now list tasks and verify the 3 headings became 3 tasks.
    let (code2, stdout2, _) = run(dir.path(), &["task", "list", "--json"]);
    assert_eq!(code2, 0);
    let tasks: serde_json::Value = serde_json::from_str(&stdout2).expect("valid JSON");
    let arr = tasks.as_array().expect("array");
    assert_eq!(
        arr.len(),
        3,
        "expected 3 tasks from fixture, got {}",
        arr.len()
    );

    let titles: Vec<&str> = arr
        .iter()
        .map(|t| t["title"].as_str().unwrap_or(""))
        .collect();
    assert!(
        titles.iter().any(|t| t.contains("failing test")),
        "expected task with 'failing test' in title, got {titles:?}"
    );
    assert!(
        titles.iter().any(|t| t.contains("feature")),
        "expected task with 'feature' in title, got {titles:?}"
    );
    assert!(
        titles.iter().any(|t| t.contains("Refactor")),
        "expected task with 'Refactor' in title, got {titles:?}"
    );
}

// ── task_pull_github_imports_issues ──────────────────────────────────────────

const GH_ISSUE_JSON: &str = r#"[
  {"number": 42, "title": "Fix the thing", "body": "Some details", "labels": []},
  {"number": 43, "title": "Another issue", "body": "", "labels": []}
]"#;

#[test]
fn task_pull_github_imports_issues() {
    let dir = TempDir::new().unwrap();

    // Create a fake `gh` binary that returns fixture JSON.
    let fake_gh = dir.path().join("gh");
    let script = format!(
        "#!/bin/sh\nprintf '%s' '{}'\nexit 0\n",
        GH_ISSUE_JSON.replace('\'', "'\\''")
    );
    std::fs::write(&fake_gh, &script).unwrap();
    std::fs::set_permissions(&fake_gh, std::fs::Permissions::from_mode(0o755)).unwrap();
    let path_with_fake = format!(
        "{}:{}",
        dir.path().to_str().unwrap(),
        std::env::var("PATH").unwrap_or_default()
    );

    let out = Command::new(godmode_bin())
        .args(["task", "pull", "--github", "--json"])
        .current_dir(dir.path())
        .env("PATH", &path_with_fake)
        .output()
        .expect("failed to run godmode");
    let code = out.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    assert_eq!(
        code,
        0,
        "task pull --github should exit 0; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // Should report how many were imported.
    let v: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    assert_eq!(v["ok"], true);
    assert_eq!(v["imported"], 2);
}

#[test]
fn task_pull_github_idempotent() {
    let dir = TempDir::new().unwrap();

    let fake_gh = dir.path().join("gh");
    let script = format!(
        "#!/bin/sh\nprintf '%s' '{}'\nexit 0\n",
        GH_ISSUE_JSON.replace('\'', "'\\''")
    );
    std::fs::write(&fake_gh, &script).unwrap();
    std::fs::set_permissions(&fake_gh, std::fs::Permissions::from_mode(0o755)).unwrap();
    let path_with_fake = format!(
        "{}:{}",
        dir.path().to_str().unwrap(),
        std::env::var("PATH").unwrap_or_default()
    );

    // First pull
    let out1 = Command::new(godmode_bin())
        .args(["task", "pull", "--github", "--json"])
        .current_dir(dir.path())
        .env("PATH", &path_with_fake)
        .output()
        .expect("run 1");
    assert_eq!(out1.status.code().unwrap_or(-1), 0);

    // Second pull — same data, should still exit 0 with imported=0 (skipped)
    let out2 = Command::new(godmode_bin())
        .args(["task", "pull", "--github", "--json"])
        .current_dir(dir.path())
        .env("PATH", &path_with_fake)
        .output()
        .expect("run 2");
    assert_eq!(out2.status.code().unwrap_or(-1), 0);
    let stdout2 = String::from_utf8_lossy(&out2.stdout).into_owned();
    let v2: serde_json::Value = serde_json::from_str(stdout2.trim()).expect("valid JSON run 2");
    assert_eq!(v2["imported"], 0, "second pull should import 0 new tasks");
}

// ── plan_depends_on_annotation_overrides_sequential ──────────────────────────

#[test]
fn plan_depends_on_annotation_overrides_sequential() {
    let md = r#"
### Task 1: Alpha
### Task 2: Beta
### Task 3: Gamma
**Depends-on**: `t1`
"#;
    let tasks = godmode_core::plan::parse(md).unwrap();
    assert_eq!(tasks[2].depends_on, vec!["t1"]);
    // t1 and t2 use default sequential — t1 has no deps, t2 depends on t1
    assert!(tasks[0].depends_on.is_empty());
    assert_eq!(tasks[1].depends_on, vec!["t1"]);
}

// ── dispatch_json_shape ──────────────────────────────────────────────────────

#[test]
fn dispatch_json_shape() {
    let dir = TempDir::new().unwrap();
    // Ingest a plan so there is something to dispatch.
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/simple-plan.md");
    run(dir.path(), &["plan", "ingest", fixture.to_str().unwrap()]);

    let (code, stdout, _stderr) = run(dir.path(), &["dispatch", "--json"]);
    assert_eq!(code, 0, "dispatch --json should exit 0");
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("dispatch --json output should be valid JSON");
    assert!(
        parsed.is_array(),
        "dispatch --json output should be a JSON array, got: {stdout}"
    );
}

// ── status_exits_0 ───────────────────────────────────────────────────────────

#[test]
fn status_exits_0() {
    let dir = TempDir::new().unwrap();
    let (code, _stdout, _stderr) = run(dir.path(), &["status"]);
    assert_eq!(code, 0, "status should exit 0");
}

// ── status_json_shape ────────────────────────────────────────────────────────

#[test]
fn status_json_shape() {
    let dir = TempDir::new().unwrap();
    let (code, stdout, _stderr) = run(dir.path(), &["status", "--json"]);
    assert_eq!(code, 0, "status --json should exit 0");
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("status --json output should be valid JSON");
    assert!(
        parsed.is_object(),
        "status --json output should be a JSON object"
    );
    let obj = parsed.as_object().unwrap();
    for key in &["done", "running", "pending", "blocked"] {
        assert!(obj.contains_key(*key), "status --json missing key: {key}");
    }
}

// ── context_exits_0 ──────────────────────────────────────────────────────────

#[test]
fn context_exits_0() {
    let dir = TempDir::new().unwrap();
    let (code, _stdout, _stderr) = run(dir.path(), &["context"]);
    assert_eq!(code, 0, "context should exit 0 even on empty graph");
}

// ── context_json_shape ───────────────────────────────────────────────────────

#[test]
fn context_json_shape() {
    let dir = TempDir::new().unwrap();
    let (code, stdout, _stderr) = run(dir.path(), &["context", "--json"]);
    assert_eq!(code, 0, "context --json should exit 0");
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("context --json output should be valid JSON");
    assert!(parsed.is_object(), "context --json should be a JSON object");
    let obj = parsed.as_object().unwrap();
    for key in &["git_root", "project", "running", "pending_count", "blocked"] {
        assert!(obj.contains_key(*key), "context --json missing key: {key}");
    }
    assert!(obj["running"].is_array(), "running should be an array");
    assert!(obj["blocked"].is_array(), "blocked should be an array");
}

// ── task_add_appears_in_list ─────────────────────────────────────────────────

#[test]
fn task_add_appears_in_list() {
    let dir = TempDir::new().unwrap();
    let (code, _stdout, _stderr) = run(
        dir.path(),
        &["task", "add", "Test task title", "--id", "my-task"],
    );
    assert_eq!(code, 0, "task add should exit 0");

    let (code2, stdout2, _) = run(dir.path(), &["task", "list", "--json"]);
    assert_eq!(code2, 0, "task list should exit 0 after add");
    let tasks: serde_json::Value = serde_json::from_str(&stdout2).expect("valid JSON");
    let arr = tasks.as_array().expect("array");
    assert_eq!(arr.len(), 1, "expected 1 task after add");
    assert_eq!(arr[0]["id"].as_str().unwrap_or(""), "my-task");
    assert_eq!(arr[0]["title"].as_str().unwrap_or(""), "Test task title");
}

// ── task_start_shows_running ─────────────────────────────────────────────────

#[test]
fn task_start_shows_running() {
    let dir = TempDir::new().unwrap();
    run(dir.path(), &["task", "add", "Task one", "--id", "t1"]);

    let (code, _stdout, _stderr) = run(dir.path(), &["task", "start", "t1"]);
    assert_eq!(code, 0, "task start should exit 0");

    let (_, stdout, _) = run(dir.path(), &["task", "list", "--json"]);
    let tasks: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    let arr = tasks.as_array().expect("array");
    assert_eq!(arr[0]["status"].as_str().unwrap_or(""), "running");
}

// ── task_done_removes_from_running ───────────────────────────────────────────

#[test]
fn task_done_removes_from_running() {
    let dir = TempDir::new().unwrap();
    run(dir.path(), &["task", "add", "Task one", "--id", "t1"]);
    run(dir.path(), &["task", "start", "t1"]);

    let (code, _stdout, _stderr) = run(dir.path(), &["task", "done", "t1"]);
    assert_eq!(code, 0, "task done should exit 0");

    let (_, stdout, _) = run(dir.path(), &["task", "list", "--json"]);
    let tasks: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    let arr = tasks.as_array().expect("array");
    assert_eq!(arr[0]["status"].as_str().unwrap_or(""), "done");
}

// ── task_block_unblock_roundtrip ─────────────────────────────────────────────

#[test]
fn task_block_unblock_roundtrip() {
    let dir = TempDir::new().unwrap();
    run(dir.path(), &["task", "add", "Task one", "--id", "t1"]);

    let (code_b, _stdout, _stderr) = run(dir.path(), &["task", "block", "t1", "waiting on dep"]);
    assert_eq!(code_b, 0, "task block should exit 0");
    let (_, stdout_b, _) = run(dir.path(), &["task", "list", "--json"]);
    let tasks_b: serde_json::Value = serde_json::from_str(&stdout_b).expect("valid JSON");
    assert_eq!(
        tasks_b.as_array().unwrap()[0]["status"]
            .as_str()
            .unwrap_or(""),
        "blocked"
    );

    let (code_u, _stdout, _stderr) = run(dir.path(), &["task", "unblock", "t1"]);
    assert_eq!(code_u, 0, "task unblock should exit 0");
    let (_, stdout_u, _) = run(dir.path(), &["task", "list", "--json"]);
    let tasks_u: serde_json::Value = serde_json::from_str(&stdout_u).expect("valid JSON");
    assert_eq!(
        tasks_u.as_array().unwrap()[0]["status"]
            .as_str()
            .unwrap_or(""),
        "pending"
    );
}

// ── task_remove ──────────────────────────────────────────────────────────────

#[test]
fn task_remove() {
    let dir = TempDir::new().unwrap();
    run(dir.path(), &["task", "add", "Task one", "--id", "t1"]);

    let (code, _stdout, _stderr) = run(dir.path(), &["task", "remove", "t1"]);
    assert_eq!(code, 0, "task remove should exit 0");

    let (code2, _stdout2, _) = run(dir.path(), &["task", "list", "--json"]);
    assert_eq!(code2, 0, "task list on empty graph should exit 0");
}

// ── task_clear_all ───────────────────────────────────────────────────────────

#[test]
fn task_clear_all() {
    let dir = TempDir::new().unwrap();
    run(dir.path(), &["task", "add", "Task one", "--id", "t1"]);
    run(dir.path(), &["task", "add", "Task two", "--id", "t2"]);

    let (_, stdout_pre, _) = run(dir.path(), &["task", "list", "--json"]);
    let pre: serde_json::Value = serde_json::from_str(&stdout_pre).expect("valid JSON");
    assert_eq!(pre.as_array().unwrap().len(), 2);

    let (code, _stdout, _stderr) = run(dir.path(), &["task", "clear", "--all"]);
    assert_eq!(code, 0, "task clear --all should exit 0");

    let (code2, _stdout2, _) = run(dir.path(), &["task", "list", "--json"]);
    assert_eq!(
        code2, 0,
        "task list on empty graph should exit 0 after clear"
    );
}

// ── handoff_with_running_task_exits_0_and_warns ──────────────────────────────

#[test]
fn handoff_with_running_task_exits_0_and_warns() {
    let dir = TempDir::new().unwrap();
    run(dir.path(), &["task", "add", "Task one", "--id", "t1"]);
    run(dir.path(), &["task", "start", "t1"]);

    let (code, _stdout, stderr) = run(dir.path(), &["handoff"]);
    assert_eq!(code, 0, "handoff exits 0 (warns but does not block)");
    assert!(
        stderr.contains("running") || _stdout.contains("running"),
        "handoff should mention running tasks"
    );
}

// ── dispatch_empty_exits_2 ───────────────────────────────────────────────────

#[test]
fn dispatch_empty_exits_2() {
    let dir = TempDir::new().unwrap();
    let (code, _stdout, _stderr) = run(dir.path(), &["dispatch", "--json"]);
    assert_eq!(
        code, 2,
        "dispatch on empty graph should exit 2 (empty result)"
    );
}
