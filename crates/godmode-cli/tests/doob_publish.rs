use std::os::unix::fs::PermissionsExt;
use std::process::Command;

fn godmode_bin() -> std::path::PathBuf {
    std::env::var("CARGO_BIN_EXE_godmode")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .join("target/debug/godmode")
        })
}

#[test]
fn task_push_publishes_once_and_persists_doob_provenance() {
    let dir = tempfile::TempDir::new().unwrap();
    let add = Command::new(godmode_bin())
        .args(["task", "add", "Publish me", "--id", "t1"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(add.status.success());

    let bin_dir = dir.path().join("bin");
    std::fs::create_dir(&bin_dir).unwrap();
    let log = dir.path().join("doob.log");
    let script = bin_dir.join("doob");
    std::fs::write(
        &script,
        format!(
            "#!/bin/sh\nprintf %s\\n called >> {}\nprintf %s \x27[{{\"id\":\"remote-1\"}}]\x27\n",
            log.display()
        ),
    )
    .unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    let path = format!(
        "{}:{}",
        bin_dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );

    for expected in [1, 0] {
        let output = Command::new(godmode_bin())
            .args(["--json", "task", "push", "--project", "godmode"])
            .current_dir(dir.path())
            .env("PATH", &path)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "push failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(json["published"], expected);
    }

    let calls = std::fs::read_to_string(log).unwrap();
    assert_eq!(calls.lines().count(), 1);
    let yaml = std::fs::read_to_string(dir.path().join(".ctx/godmode/tasks.yaml")).unwrap();
    assert!(yaml.contains("provenance:"), "yaml: {yaml}");
    assert!(yaml.contains("id: remote-1"), "yaml: {yaml}");
}
