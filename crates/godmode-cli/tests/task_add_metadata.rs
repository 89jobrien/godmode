use std::process::Command;

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

fn run_godmode(dir: &tempfile::TempDir, args: &[&str]) -> std::process::Output {
    Command::new(godmode_bin())
        .args(args)
        .current_dir(dir.path())
        .output()
        .expect("run godmode")
}

#[test]
fn task_add_persists_metadata_and_tag_order_in_yaml_and_json() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let output = run_godmode(
        &dir,
        &[
            "task",
            "add",
            "Ship metadata",
            "--id",
            "t7",
            "--notes",
            "Preserve this note",
            "--run",
            "cargo nextest run -p godmode-cli",
            "--priority",
            "high",
            "--tag",
            "cli",
            "--tag",
            "metadata",
        ],
    );
    assert!(
        output.status.success(),
        "task add failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let yaml = std::fs::read_to_string(dir.path().join(".ctx/godmode/tasks.yaml"))
        .expect("persisted task graph");
    assert!(yaml.contains("notes: Preserve this note"), "yaml:\n{yaml}");
    assert!(
        yaml.contains("run: cargo nextest run -p godmode-cli"),
        "yaml:\n{yaml}"
    );
    assert!(yaml.contains("priority: high"), "yaml:\n{yaml}");
    let cli_tag = yaml.find("- cli").expect("cli tag in yaml");
    let metadata_tag = yaml.find("- metadata").expect("metadata tag in yaml");
    assert!(cli_tag < metadata_tag, "tag order changed in yaml:\n{yaml}");

    let list = run_godmode(&dir, &["--json", "task", "list"]);
    assert!(list.status.success(), "task list failed");
    let tasks: serde_json::Value = serde_json::from_slice(&list.stdout).expect("task list json");
    let task = &tasks[0];
    assert_eq!(task["notes"], "Preserve this note");
    assert_eq!(task["run"], "cargo nextest run -p godmode-cli");
    assert_eq!(task["priority"], "high");
    assert_eq!(task["tags"], serde_json::json!(["cli", "metadata"]));
}

#[test]
fn task_add_without_metadata_retains_existing_defaults() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let output = run_godmode(&dir, &["task", "add", "Minimal task", "--id", "t1"]);
    assert!(output.status.success());

    let list = run_godmode(&dir, &["--json", "task", "list"]);
    let tasks: serde_json::Value = serde_json::from_slice(&list.stdout).expect("task list json");
    let task = &tasks[0];
    assert_eq!(task["notes"], "");
    assert!(task.get("run").is_none());
    assert!(task.get("priority").is_none());
    assert!(task.get("tags").is_none());
}

#[test]
fn task_add_help_lists_metadata_options_and_priority_values() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let output = run_godmode(&dir, &["task", "add", "--help"]);
    assert!(output.status.success());
    let help = String::from_utf8_lossy(&output.stdout);
    for option in ["--notes", "--run", "--priority", "--tag"] {
        assert!(help.contains(option), "missing {option} in help:\n{help}");
    }
    assert!(help.contains("--tag <TAG>"), "help: {help}");
    assert!(help.contains("high"), "help:\n{help}");
    assert!(help.contains("normal"), "help:\n{help}");
    assert!(help.contains("low"), "help:\n{help}");
}

#[test]
fn task_add_rejects_unsupported_priority() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let output = run_godmode(
        &dir,
        &["task", "add", "Invalid priority", "--priority", "urgent"],
    );
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("urgent"), "stderr: {stderr}");
}
