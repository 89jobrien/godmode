use std::path::Path;
use std::process::{Command, Output};

fn godmode_bin() -> std::path::PathBuf {
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_godmode") {
        return std::path::PathBuf::from(path);
    }
    let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .expect("crate parent")
        .parent()
        .expect("workspace root")
        .join("target/debug/godmode")
}

fn run_godmode(dir: &Path, args: &[&str]) -> Output {
    Command::new(godmode_bin())
        .args(args)
        .current_dir(dir)
        .output()
        .expect("run godmode")
}

fn write_plan(path: &Path, first: &str, second: &str) {
    std::fs::write(
        path,
        format!("# Plan\n\n## Tasks\n\n### Task 1: {first}\n\n### Task 2: {second}\n"),
    )
    .expect("write plan");
}

#[test]
fn multiple_plan_ingests_use_distinct_id_namespaces() {
    let dir = tempfile::tempdir().expect("tempdir");
    let first = dir.path().join("first-plan.md");
    let second = dir.path().join("second-plan.md");
    write_plan(&first, "First alpha", "First beta");
    write_plan(&second, "Second alpha", "Second beta");

    let first_output = run_godmode(
        dir.path(),
        &["--json", "plan", "ingest", first.to_str().unwrap()],
    );
    assert!(first_output.status.success());

    let second_output = run_godmode(
        dir.path(),
        &["--json", "plan", "ingest", second.to_str().unwrap()],
    );
    assert!(second_output.status.success());
    let report: serde_json::Value =
        serde_json::from_slice(&second_output.stdout).expect("ingest report json");
    assert_eq!(report["parsed"], 2);
    assert_eq!(report["added"], 2);
    assert_eq!(report["skipped"], 0);

    let list = run_godmode(dir.path(), &["--json", "task", "list"]);
    assert!(list.status.success());
    let tasks: serde_json::Value = serde_json::from_slice(&list.stdout).expect("task list json");
    let tasks = tasks.as_array().expect("task list array");
    assert_eq!(tasks.len(), 4);
    assert_eq!(tasks[0]["id"], "t1");
    assert_eq!(tasks[1]["id"], "t2");
    assert_eq!(tasks[2]["id"], "second-plan-t1");
    assert_eq!(tasks[3]["id"], "second-plan-t2");
    assert_eq!(
        tasks[3]["depends_on"],
        serde_json::json!(["second-plan-t1"])
    );

    let repeated = run_godmode(
        dir.path(),
        &["--json", "plan", "ingest", second.to_str().unwrap()],
    );
    assert!(repeated.status.success());
    let report: serde_json::Value =
        serde_json::from_slice(&repeated.stdout).expect("repeat report json");
    assert_eq!(report["parsed"], 2);
    assert_eq!(report["added"], 0);
    assert_eq!(report["skipped"], 2);
}

#[test]
fn agent_dispatch_ingest_uses_distinct_id_namespace() {
    let dir = tempfile::tempdir().expect("tempdir");
    let first = dir.path().join("first-plan.md");
    let second = dir.path().join("dispatch-plan.md");
    write_plan(&first, "First alpha", "First beta");
    write_plan(&second, "Dispatch alpha", "Dispatch beta");

    let initial = run_godmode(
        dir.path(),
        &["--json", "plan", "ingest", first.to_str().unwrap()],
    );
    assert!(initial.status.success());

    let dispatched = run_godmode(
        dir.path(),
        &[
            "--json",
            "agent",
            "dispatch",
            second.to_str().unwrap(),
            "--max",
            "1",
        ],
    );
    assert!(
        dispatched.status.success(),
        "agent dispatch failed: {}",
        String::from_utf8_lossy(&dispatched.stderr)
    );
    let report: serde_json::Value =
        serde_json::from_slice(&dispatched.stdout).expect("dispatch report json");
    assert_eq!(report["parsed"], 2);
    assert_eq!(report["ingested"], 2);
    assert_eq!(report["skipped"], 0);

    let list = run_godmode(dir.path(), &["--json", "task", "list"]);
    let tasks: serde_json::Value = serde_json::from_slice(&list.stdout).expect("task list json");
    let tasks = tasks.as_array().expect("task list array");
    assert_eq!(tasks.len(), 4);
    assert_eq!(tasks[2]["id"], "dispatch-plan-t1");
    assert_eq!(
        tasks[3]["depends_on"],
        serde_json::json!(["dispatch-plan-t1"])
    );

    let repeated = run_godmode(
        dir.path(),
        &[
            "--json",
            "agent",
            "dispatch",
            second.to_str().unwrap(),
            "--max",
            "1",
        ],
    );
    assert!(repeated.status.success());
    let report: serde_json::Value =
        serde_json::from_slice(&repeated.stdout).expect("repeat dispatch report json");
    assert_eq!(report["ingested"], 0);
    assert_eq!(report["skipped"], 2);
}

#[test]
fn plans_with_same_filename_in_different_directories_remain_distinct() {
    let dir = tempfile::tempdir().expect("tempdir");
    let first_dir = dir.path().join("first");
    let second_dir = dir.path().join("second");
    std::fs::create_dir_all(&first_dir).expect("first plan directory");
    std::fs::create_dir_all(&second_dir).expect("second plan directory");
    let first = first_dir.join("plan.md");
    let second = second_dir.join("plan.md");
    write_plan(&first, "Shared alpha", "Shared beta");
    write_plan(&second, "Shared alpha", "Shared beta");

    for path in [&first, &second] {
        let output = run_godmode(
            dir.path(),
            &["--json", "plan", "ingest", path.to_str().unwrap()],
        );
        assert!(output.status.success());
    }

    let list = run_godmode(dir.path(), &["--json", "task", "list"]);
    let tasks: serde_json::Value = serde_json::from_slice(&list.stdout).expect("task list json");
    let tasks = tasks.as_array().expect("task list array");
    assert_eq!(tasks.len(), 4);
    assert_eq!(tasks[2]["id"], "plan-t1");
    assert_eq!(tasks[3]["id"], "plan-t2");
}
