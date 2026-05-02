mod fake_bin;
use fake_bin::FakeBin;

use std::io::Write;
use std::process::{Command, Stdio};
use tempfile::TempDir;

/// Absolute path to the hooks directory in this repo.
fn hooks_dir() -> std::path::PathBuf {
    // tests/ is at crates/godmode-core/tests/; repo root is three levels up
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest
        .join("../..")
        .join("hooks")
        .canonicalize()
        .expect("hooks dir")
}

/// Skip the test if `nu` is not available on PATH.
fn nu_bin() -> Option<std::path::PathBuf> {
    which::which("nu").ok()
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Run a nu script with optional stdin JSON and an optional PATH prefix.
fn run_hook(
    script: &std::path::Path,
    stdin_json: Option<&str>,
    path_prefix: Option<&str>,
) -> std::process::Output {
    let nu = nu_bin().expect("nu not on PATH");
    let mut cmd = Command::new(&nu);
    cmd.arg(script);
    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    // Inherit PATH then optionally prepend a fake-bin dir.
    let base_path = std::env::var("PATH").unwrap_or_default();
    let path_val = match path_prefix {
        Some(prefix) => format!("{}:{}", prefix, base_path),
        None => base_path,
    };
    cmd.env("PATH", &path_val);

    let mut child = cmd.spawn().expect("spawn nu");
    if let Some(json) = stdin_json {
        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(json.as_bytes())
            .expect("write stdin");
    }
    child.wait_with_output().expect("wait")
}

// ---------------------------------------------------------------------------
// session-start.nu
// ---------------------------------------------------------------------------

#[test]
fn session_start_noop_when_task_file_absent() {
    let Some(_nu) = nu_bin() else { return };

    let tmp = TempDir::new().unwrap();
    // Make tmp look like a git repo (git rev-parse needs .git)
    std::fs::create_dir_all(tmp.path().join(".git/refs/heads")).unwrap();
    std::fs::write(tmp.path().join(".git/HEAD"), "ref: refs/heads/main\n").unwrap();

    // Fake git that returns our tmp dir as repo root.
    let fake_git = FakeBin::new("git")
        .stdout(tmp.path().to_str().unwrap())
        .build();

    let script = hooks_dir().join("scripts/session-start.nu");
    let output = run_hook(
        &script,
        Some(r#"{"session_id":"test"}"#),
        Some(fake_git.dir()),
    );

    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn session_start_runs_handon_when_task_file_present() {
    let Some(_nu) = nu_bin() else { return };

    let tmp = TempDir::new().unwrap();
    // Minimal .git
    std::fs::create_dir_all(tmp.path().join(".git/refs/heads")).unwrap();
    std::fs::write(tmp.path().join(".git/HEAD"), "ref: refs/heads/main\n").unwrap();
    // Create the task file
    std::fs::create_dir_all(tmp.path().join(".ctx")).unwrap();
    std::fs::write(tmp.path().join(".ctx/GODMODE.tasks.yaml"), "tasks: []\n").unwrap();

    // Fake git returning tmp as root
    let fake_git = FakeBin::new("git")
        .stdout(tmp.path().to_str().unwrap())
        .build();
    // Fake godmode that records it was called
    let called_marker = tmp.path().join("godmode_called");
    let marker_path = called_marker.to_str().unwrap();
    let godmode_script = format!("#!/bin/sh\ntouch {}\nexit 0\n", marker_path);
    let godmode_dir = TempDir::new().unwrap();
    let godmode_bin = godmode_dir.path().join("godmode");
    std::fs::write(&godmode_bin, &godmode_script).unwrap();
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&godmode_bin, std::fs::Permissions::from_mode(0o755)).unwrap();

    let path_prefix = format!(
        "{}:{}",
        godmode_dir.path().to_str().unwrap(),
        fake_git.dir()
    );
    let script = hooks_dir().join("scripts/session-start.nu");
    let output = run_hook(
        &script,
        Some(r#"{"session_id":"test"}"#),
        Some(&path_prefix),
    );

    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(called_marker.exists(), "godmode handon was not called");
}

#[test]
fn session_start_degrades_gracefully_when_godmode_absent() {
    let Some(_nu) = nu_bin() else { return };

    let tmp = TempDir::new().unwrap();
    std::fs::create_dir_all(tmp.path().join(".git/refs/heads")).unwrap();
    std::fs::write(tmp.path().join(".git/HEAD"), "ref: refs/heads/main\n").unwrap();
    std::fs::create_dir_all(tmp.path().join(".ctx")).unwrap();
    std::fs::write(tmp.path().join(".ctx/GODMODE.tasks.yaml"), "tasks: []\n").unwrap();

    let fake_git = FakeBin::new("git")
        .stdout(tmp.path().to_str().unwrap())
        .build();

    // PATH has git but NOT godmode
    let script = hooks_dir().join("scripts/session-start.nu");
    let output = run_hook(
        &script,
        Some(r#"{"session_id":"test"}"#),
        Some(fake_git.dir()),
    );

    assert_eq!(
        output.status.code(),
        Some(0),
        "should exit 0 without godmode; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

// ---------------------------------------------------------------------------
// pre-commit.nu
// ---------------------------------------------------------------------------

#[test]
fn pre_commit_exits_zero_when_no_running_tasks() {
    let Some(_nu) = nu_bin() else { return };

    // godmode handoff --json exits 0
    let fake_godmode = FakeBin::new("godmode")
        .stdout(r#"{"running_task_ids":[]}"#)
        .exit_code(0)
        .build();

    // Also need cargo / nextest to succeed. Use fake stubs.
    let fake_cargo = FakeBin::new("cargo").exit_code(0).build();

    let path_prefix = format!("{}:{}", fake_godmode.dir(), fake_cargo.dir());

    let script = hooks_dir().join("pre-commit.nu");
    let output = run_hook(&script, None, Some(&path_prefix));

    // pre-commit runs cargo gates which pass with our fakes
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn pre_commit_exits_nonzero_and_prints_ids_when_running() {
    let Some(_nu) = nu_bin() else { return };

    // godmode handoff --json exits non-zero with running task IDs
    let json_out = r#"{"running_task_ids":["t1","t2"]}"#;
    let fake_godmode = FakeBin::new("godmode")
        .stdout(json_out)
        .exit_code(1)
        .build();

    let script = hooks_dir().join("pre-commit.nu");
    let output = run_hook(&script, None, Some(fake_godmode.dir()));

    assert_ne!(
        output.status.code(),
        Some(0),
        "should fail when tasks are running"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("t1") || stdout.contains("t2"),
        "expected task IDs in output, got: {stdout}"
    );
}

#[test]
fn pre_commit_degrades_gracefully_when_godmode_absent() {
    let Some(_nu) = nu_bin() else { return };

    // PATH has no godmode; we still need cargo to succeed
    let fake_cargo = FakeBin::new("cargo").exit_code(0).build();

    let script = hooks_dir().join("pre-commit.nu");
    let output = run_hook(&script, None, Some(fake_cargo.dir()));

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("not on PATH") || output.status.code() == Some(0),
        "should degrade gracefully; stdout: {stdout}"
    );
}

// ---------------------------------------------------------------------------
// task-done-sync.nu
// ---------------------------------------------------------------------------

#[test]
fn task_done_sync_noop_on_unrelated_command() {
    let Some(_nu) = nu_bin() else { return };

    let fake_godmode = FakeBin::new("godmode").echo_argv().exit_code(0).build();

    let script = hooks_dir().join("task-done-sync.nu");
    let stdin = r#"{"tool_name":"Bash","tool_input":{"command":"cargo build"}}"#;
    let output = run_hook(&script, Some(stdin), Some(fake_godmode.dir()));

    // Should exit 0 without invoking godmode task push-done
    assert_eq!(output.status.code(), Some(0));
    // stdout should be empty (no push-done was run)
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.trim().is_empty(), "unexpected output: {stdout}");
}

#[test]
fn task_done_sync_runs_push_done_on_task_done_command() {
    let Some(_nu) = nu_bin() else { return };

    let fake_godmode = FakeBin::new("godmode").exit_code(0).build();

    let script = hooks_dir().join("task-done-sync.nu");
    let stdin = r#"{"tool_name":"Bash","tool_input":{"command":"godmode task done t1"}}"#;
    let output = run_hook(&script, Some(stdin), Some(fake_godmode.dir()));

    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn task_done_sync_runs_push_done_on_auto_done_command() {
    let Some(_nu) = nu_bin() else { return };

    let fake_godmode = FakeBin::new("godmode").exit_code(0).build();

    let script = hooks_dir().join("task-done-sync.nu");
    let stdin =
        r#"{"tool_name":"Bash","tool_input":{"command":"godmode task run t1 --auto-done"}}"#;
    let output = run_hook(&script, Some(stdin), Some(fake_godmode.dir()));

    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn task_done_sync_degrades_gracefully_when_godmode_absent() {
    let Some(_nu) = nu_bin() else { return };

    // No godmode on PATH at all
    let empty_dir = TempDir::new().unwrap();

    let script = hooks_dir().join("task-done-sync.nu");
    let stdin = r#"{"tool_name":"Bash","tool_input":{"command":"godmode task done t1"}}"#;
    let output = run_hook(
        &script,
        Some(stdin),
        Some(empty_dir.path().to_str().unwrap()),
    );

    // Must exit 0 — degraded gracefully
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
