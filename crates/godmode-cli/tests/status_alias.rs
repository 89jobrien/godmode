//! CLI regression coverage for legacy task-status aliases.

use std::process::Command;

fn godmode_bin() -> std::path::PathBuf {
    std::env::var("CARGO_BIN_EXE_godmode")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/debug/godmode")
        })
}

#[test]
fn graph_commands_accept_active_status_from_existing_files() {
    let temp = tempfile::tempdir().unwrap();
    let state_dir = temp.path().join(".ctx/godmode");
    std::fs::create_dir_all(&state_dir).unwrap();
    std::fs::write(
        state_dir.join("tasks.yaml"),
        concat!(
            "tasks:\n",
            "  - id: existing\n    title: Existing work\n    status: active\n",
            "  - id: pending\n    title: Pending work\n    status: pending\n",
        ),
    )
    .unwrap();
    let plan = temp.path().join("plan.md");
    std::fs::write(&plan, "### Task 1: Follow-up\n").unwrap();

    let commands = [
        vec!["handon"],
        vec!["task", "next", "--json"],
        vec!["plan", "ingest", plan.to_str().unwrap()],
    ];
    for args in commands {
        let result = Command::new(godmode_bin())
            .args(&args)
            .current_dir(temp.path())
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "godmode {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&result.stderr)
        );
    }

    let saved = std::fs::read_to_string(state_dir.join("tasks.yaml")).unwrap();
    assert!(saved.contains("status: running"), "got: {saved}");
    assert!(!saved.contains("status: active"), "got: {saved}");
}

#[test]
fn task_driven_development_producers_use_canonical_statuses() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let skill =
        std::fs::read_to_string(root.join("skills/task-driven-development/SKILL.md")).unwrap();
    let schema = std::fs::read_to_string(
        root.join("skills/task-driven-development/helpers/task-schema.yaml"),
    )
    .unwrap();
    let runner =
        std::fs::read_to_string(root.join("skills/task-driven-development/helpers/task-runner.rs"))
            .unwrap();

    for (name, content) in [("skill", skill.as_str()), ("schema", schema.as_str())] {
        assert!(
            content.contains("status: running"),
            "{name} must emit running"
        );
        assert!(
            content.contains("status: blocked"),
            "{name} must emit blocked"
        );
        assert!(
            !content.contains("status: active"),
            "{name} must not emit the legacy active alias"
        );
        assert!(
            !content.contains("status: failed"),
            "{name} must not emit the unsupported failed status"
        );
    }

    assert!(runner.contains("task.status = Status::Running;"));
    assert!(runner.contains("task.status = Status::Blocked;"));
    assert!(!runner.contains("task.status = Status::Active;"));
    assert!(!runner.contains("task.status = Status::Failed;"));
}
