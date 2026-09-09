//! Behavioral integration tests for hook commands.
use std::os::unix::fs::PermissionsExt;
use std::process::{Command, Stdio};
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
fn script(path: &std::path::Path, body: &str) {
    std::fs::write(path, body).unwrap();
    let mut p = std::fs::metadata(path).unwrap().permissions();
    p.set_mode(0o755);
    std::fs::set_permissions(path, p).unwrap();
}
#[test]
fn hook_list_renders_entries_and_rejects_malformed_json() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::create_dir(temp.path().join("hooks")).unwrap();
    std::fs::write(
        temp.path().join("hooks/hooks.json"),
        r#"{"hooks":{"Stop":[{"matcher":"*","hooks":[{"command":"stop.nu"}]}]}}"#,
    )
    .unwrap();
    let output = run(temp.path(), &["--json", "hook", "list"]);
    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value[0]["event"], "Stop");
    assert_eq!(value[0]["script"], "stop.nu");
    std::fs::write(temp.path().join("hooks/hooks.json"), "{").unwrap();
    let output = run(temp.path(), &["hook", "list"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("hooks/hooks.json"));
}
#[test]
fn hook_log_renders_tail_and_reports_read_errors() {
    let temp = tempfile::tempdir().unwrap();
    let traces = temp.path().join(".ctx/godmode/traces");
    std::fs::create_dir_all(&traces).unwrap();
    std::fs::write(traces.join("hooks.log"), concat!("{\"hook\":\"one\",\"event\":\"Stop\",\"exit_code\":0,\"stderr\":\"\",\"ts\":\"t1\"}\n","{\"hook\":\"two\",\"event\":\"Stop\",\"exit_code\":1,\"stderr\":\"bad\",\"ts\":\"t2\"}\n")).unwrap();
    let output = run(temp.path(), &["hook", "log", "--tail", "1"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("two"));
    assert!(!stdout.contains("one"));
    std::fs::remove_file(traces.join("hooks.log")).unwrap();
    std::fs::create_dir(traces.join("hooks.log")).unwrap();
    let output = run(temp.path(), &["hook", "log"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("hook log"));
}
#[test]
fn hook_test_reports_success_nonzero_and_spawn_errors() {
    let temp = tempfile::tempdir().unwrap();
    let ok = temp.path().join("ok.sh");
    script(&ok, "#!/bin/sh\nread input\nprintf %s \"$input\"\n");
    let output = Command::new(bin())
        .args(["--json", "hook", "test"])
        .arg(&ok)
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["exit_code"], 0);
    assert!(value["stdout"].as_str().unwrap().contains("tool_input"));
    let bad = temp.path().join("bad.sh");
    script(&bad, "#!/bin/sh\necho failed >&2\nexit 7\n");
    let output = Command::new(bin())
        .args(["hook", "test"])
        .arg(&bad)
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("failed"));
    let empty = temp.path().join("empty");
    std::fs::create_dir(&empty).unwrap();
    let output = Command::new(bin())
        .args(["hook", "test"])
        .arg(&ok)
        .env("PATH", &empty)
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("failed to spawn hook script"));
}
#[test]
fn hook_migrate_runs_in_order_and_stops_on_nonzero() {
    let temp = tempfile::tempdir().unwrap();
    let migrations = temp.path().join("hooks/migrations");
    std::fs::create_dir_all(&migrations).unwrap();
    let record = temp.path().join("record");
    script(
        &migrations.join("002-second.sh"),
        &format!("#!/bin/sh\nprintf 2 >> {}\n", record.display()),
    );
    script(
        &migrations.join("001-first.sh"),
        &format!("#!/bin/sh\nprintf 1 >> {}\n", record.display()),
    );
    let output = run(temp.path(), &["--json", "hook", "migrate"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(std::fs::read_to_string(&record).unwrap(), "12");
    script(
        &migrations.join("003-fail.sh"),
        "#!/bin/sh\necho migration-broke >&2\nexit 4\n",
    );
    let output = run(temp.path(), &["hook", "migrate"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("003-fail.sh"));
    assert!(stderr.contains("migration-broke"));
}
#[test]
fn hook_run_rejects_malformed_input_and_unknown_builtins() {
    use std::io::Write as _;
    let temp = tempfile::tempdir().unwrap();
    let mut child = Command::new(bin())
        .args(["hook", "run", "auto-block"])
        .current_dir(temp.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"{").unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("parsing hook stdin"));
    let output = Command::new(bin())
        .args(["hook", "run", "not-a-hook"])
        .current_dir(temp.path())
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unknown built-in hook"));
}
#[test]
fn hook_auto_block_persists_through_session() {
    use std::io::Write as _;
    let temp = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(temp.path().join(".ctx/godmode")).unwrap();
    std::fs::write(
        temp.path().join(".ctx/godmode/tasks.yaml"),
        "tasks:\n- id: t1\n  title: task\n  status: running\n  depends_on: []\n  notes: \"\"\n",
    )
    .unwrap();
    let mut child = Command::new(bin())
        .args(["hook", "run", "auto-block"])
        .current_dir(temp.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(br#"{"tool_input":{"command":"cargo test"},"tool_result":{"exit_code":1,"stdout":"FAILED boom"}}"#).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let graph = std::fs::read_to_string(temp.path().join(".ctx/godmode/tasks.yaml")).unwrap();
    assert!(graph.contains("status: blocked"));
}
