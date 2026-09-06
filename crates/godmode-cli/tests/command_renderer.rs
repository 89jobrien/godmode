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
