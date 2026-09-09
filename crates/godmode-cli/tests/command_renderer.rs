//! CLI integration tests for command rendering.

use std::process::Command;

fn godmode_bin() -> std::path::PathBuf {
    std::env::var("CARGO_BIN_EXE_godmode")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/debug/godmode")
        })
}

#[test]
fn command_renderer_json_names_target_and_destination() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let output = temp.path().join("output");
    std::fs::create_dir_all(&source).unwrap();
    std::fs::write(
        source.join("audit.yaml"),
        "name: audit\nprompt: Audit $ARGUMENTS\n",
    )
    .unwrap();

    let result = Command::new(godmode_bin())
        .args([
            "--json",
            "command",
            "generate",
            "--target",
            "opencode",
            "--source-dir",
        ])
        .arg(&source)
        .arg("--output-dir")
        .arg(&output)
        .current_dir(temp.path())
        .output()
        .unwrap();

    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(json["target"], "opencode");
    assert_eq!(json["output_dir"], output.to_string_lossy().as_ref());
    assert!(output.join("gm-audit.md").exists());
}

fn source(root: &std::path::Path) -> std::path::PathBuf {
    let source = root.join("source");
    std::fs::create_dir_all(&source).unwrap();
    std::fs::write(
        source.join("audit.yaml"),
        "name: audit\nprompt: Audit $ARGUMENTS\n",
    )
    .unwrap();
    source
}

#[test]
fn command_generate_requires_opencode_output() {
    let temp = tempfile::tempdir().unwrap();
    let source = source(temp.path());
    let output = Command::new(godmode_bin())
        .args([
            "command",
            "generate",
            "--target",
            "opencode",
            "--source-dir",
        ])
        .arg(source)
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("--output-dir is required"));
}

#[test]
fn command_install_requires_home_without_output() {
    let temp = tempfile::tempdir().unwrap();
    let source = source(temp.path());
    let output = Command::new(godmode_bin())
        .args(["command", "install-opencode", "--source-dir"])
        .arg(source)
        .env_remove("HOME")
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("HOME is not set"));
}

#[test]
fn command_check_dry_run_and_renderer_errors_are_reported() {
    let temp = tempfile::tempdir().unwrap();
    let source = source(temp.path());
    let output_dir = temp.path().join("out");
    let missing = Command::new(godmode_bin())
        .args(["command", "generate", "--target", "claude", "--source-dir"])
        .arg(&source)
        .arg("--output-dir")
        .arg(&output_dir)
        .arg("--check")
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(!missing.status.success());
    assert!(String::from_utf8_lossy(&missing.stderr).contains("missing generated command"));
    let dry = Command::new(godmode_bin())
        .args(["--json", "command", "install-opencode", "--source-dir"])
        .arg(&source)
        .arg("--output-dir")
        .arg(&output_dir)
        .arg("--dry-run")
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(dry.status.success());
    assert!(!output_dir.exists());
    let value: serde_json::Value = serde_json::from_slice(&dry.stdout).unwrap();
    assert_eq!(value["dry_run"], true);
    std::fs::write(source.join("bad.yaml"), "name: WRONG\nprompt: x\n").unwrap();
    let error = Command::new(godmode_bin())
        .args(["command", "generate", "--target", "claude", "--source-dir"])
        .arg(&source)
        .arg("--output-dir")
        .arg(&output_dir)
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(!error.status.success());
    assert!(String::from_utf8_lossy(&error.stderr).contains("bad.yaml"));
}

#[test]
fn agent_opencode_install_reports_catalog_home_and_output_errors() {
    let temp = tempfile::tempdir().unwrap();
    let bad = temp.path().join("bad.yaml");
    std::fs::write(&bad, "projects: [").unwrap();
    let malformed = Command::new(godmode_bin())
        .args(["agent", "install-opencode", "--catalog"])
        .arg(&bad)
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(!malformed.status.success());
    assert!(String::from_utf8_lossy(&malformed.stderr).contains("bad.yaml"));
    let no_home = Command::new(godmode_bin())
        .args(["agent", "install-opencode"])
        .env_remove("HOME")
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(!no_home.status.success());
    assert!(String::from_utf8_lossy(&no_home.stderr).contains("HOME is not set"));
    let file = temp.path().join("file");
    std::fs::write(&file, "x").unwrap();
    let write_error = Command::new(godmode_bin())
        .args(["agent", "install-opencode", "--output-dir"])
        .arg(&file)
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(!write_error.status.success());
    assert!(String::from_utf8_lossy(&write_error.stderr).contains("OpenCode"));
}
