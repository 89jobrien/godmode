//! Behavioral integration tests for task dispatcher branches.
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
fn bin() -> std::path::PathBuf {
    std::env::var("CARGO_BIN_EXE_godmode")
        .map(Into::into)
        .unwrap_or_else(|_| {
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/debug/godmode")
        })
}
fn run(root: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(bin())
        .args(args)
        .current_dir(root)
        .output()
        .unwrap()
}
fn state(root: &std::path::Path, body: &str) {
    std::fs::create_dir_all(root.join(".ctx/godmode")).unwrap();
    std::fs::write(root.join(".ctx/godmode/tasks.yaml"), body).unwrap()
}
fn fake(path: &std::path::Path, name: &str, body: &str) {
    std::fs::create_dir_all(path).unwrap();
    let file = path.join(name);
    std::fs::write(&file, body).unwrap();
    let mut p = std::fs::metadata(&file).unwrap().permissions();
    p.set_mode(0o755);
    std::fs::set_permissions(file, p).unwrap()
}
fn path_with(dir: &std::path::Path) -> String {
    format!("{}:/bin:/usr/bin", dir.display())
}
#[test]
fn task_run_covers_success_auto_done_missing_command_and_nonzero() {
    let temp = tempfile::tempdir().unwrap();
    state(
        temp.path(),
        "tasks:\n- id: t1\n  title: run\n  status: running\n  depends_on: []\n  notes: \"\"\n  run: /usr/bin/true\n",
    );
    let output = run(temp.path(), &["task", "run", "t1", "--auto-done"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        std::fs::read_to_string(temp.path().join(".ctx/godmode/tasks.yaml"))
            .unwrap()
            .contains("status: done")
    );
    state(
        temp.path(),
        "tasks:\n- id: t1\n  title: run\n  status: pending\n  depends_on: []\n  notes: \"\"\n",
    );
    let output = run(temp.path(), &["task", "run", "t1"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("no `run:` field"));
    state(
        temp.path(),
        "tasks:\n- id: t1\n  title: run\n  status: pending\n  depends_on: []\n  notes: \"\"\n  run: /usr/bin/false\n",
    );
    let output = run(temp.path(), &["task", "run", "t1"]);
    assert!(!output.status.success());
    state(
        temp.path(),
        "tasks:\n- id: t1\n  title: run\n  status: pending\n  depends_on: []\n  notes: \"\"\n  run: no-such-w2-tool\n",
    );
    let output = run(temp.path(), &["task", "run", "t1"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("failed to run"));
}
#[test]
fn task_pull_doob_handles_duplicates_empty_data_and_tool_errors() {
    let temp = tempfile::tempdir().unwrap();
    state(
        temp.path(),
        "tasks:\n- id: doob-abcdefgh\n  title: old\n  status: pending\n  depends_on: []\n  notes: doob:abcdefgh1111\n",
    );
    let tools = temp.path().join("tools");
    fake(
        &tools,
        "doob",
        "#!/bin/sh\nprintf %s \"{\\\"todos\\\":[{\\\"id\\\":\\\"abcdefgh1111\\\",\\\"content\\\":\\\"old\\\",\\\"status\\\":\\\"pending\\\"},{\\\"id\\\":\\\"ijklmnop2222\\\",\\\"content\\\":\\\"new\\\",\\\"status\\\":\\\"pending\\\"}]}\"\n",
    );
    let output = Command::new(bin())
        .args(["--json", "task", "pull", "--project", "demo"])
        .env("PATH", path_with(&tools))
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["imported"], 1);
    fake(&tools, "doob", "#!/bin/sh\nprintf %s bad-json\n");
    let output = Command::new(bin())
        .args(["task", "pull", "--project", "demo"])
        .env("PATH", path_with(&tools))
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("invalid JSON"));
    let empty = temp.path().join("empty");
    std::fs::create_dir(&empty).unwrap();
    let output = Command::new(bin())
        .args(["task", "pull", "--project", "demo"])
        .env("PATH", &empty)
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("doob not found"));
}
#[test]
fn task_push_done_reports_partial_subprocess_failure() {
    let temp = tempfile::tempdir().unwrap();
    state(
        temp.path(),
        "tasks:\n- id: t1\n  title: one\n  status: done\n  depends_on: []\n  notes: doob:ok\n- id: t2\n  title: two\n  status: done\n  depends_on: []\n  notes: doob:bad\n",
    );
    let tools = temp.path().join("tools");
    fake(
        &tools,
        "doob",
        "#!/bin/sh\nif [ \"$3\" = bad ]; then echo rejected >&2; exit 9; fi\n",
    );
    let output = Command::new(bin())
        .args(["task", "push-done"])
        .env("PATH", path_with(&tools))
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("bad"));
    assert!(stderr.contains("after 1 completed task"));
}
#[test]
fn task_unblock_all_handles_blocked_and_empty_graphs() {
    let temp = tempfile::tempdir().unwrap();
    state(
        temp.path(),
        "tasks:\n- id: t1\n  title: one\n  status: blocked\n  depends_on: []\n  notes: stuck\n",
    );
    let output = run(temp.path(), &["--json", "task", "unblock-all"]);
    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["unblocked"], 1);
    let output = run(temp.path(), &["task", "unblock-all"]);
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("No blocked tasks"));
}
#[test]
fn task_apply_and_list_templates_cover_duplicates_missing_vars_and_invalid_yaml() {
    let temp = tempfile::tempdir().unwrap();
    let templates = temp.path().join("templates");
    std::fs::create_dir(&templates).unwrap();
    std::fs::write(templates.join("demo.template.yaml"),"meta:\n  name: demo\n  description: Demo\n  vars:\n    - name: title\n      required: true\ntasks:\n  - id: t1\n    title: \"{{title}}\"\n    status: pending\n    depends_on: []\n").unwrap();
    let output = run(temp.path(), &["--json", "task", "list-templates"]);
    assert!(output.status.success());
    let listed: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(listed[0]["name"], "demo");
    let output = run(temp.path(), &["task", "apply", "demo"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("title"));
    let output = run(
        temp.path(),
        &["--json", "task", "apply", "demo", "--var", "title=hello"],
    );
    assert!(output.status.success());
    let output = run(
        temp.path(),
        &["--json", "task", "apply", "demo", "--var", "title=hello"],
    );
    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["skipped"], 1);
    std::fs::write(templates.join("bad.template.yaml"), "not: [valid").unwrap();
    let output = run(temp.path(), &["task", "list-templates"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("bad.template.yaml"));
}
