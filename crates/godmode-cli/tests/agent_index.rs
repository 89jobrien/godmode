//! Integration tests for agent index generation side effects.

use std::process::Command;

fn godmode_bin() -> std::path::PathBuf {
    std::env::var("CARGO_BIN_EXE_godmode")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            manifest
                .parent()
                .expect("crate parent")
                .parent()
                .expect("workspace parent")
                .join("target/debug/godmode")
        })
}

fn write_agent(root: &std::path::Path, file: &str, name: &str) {
    let agents = root.join("agents");
    std::fs::create_dir_all(&agents).expect("agents directory");
    std::fs::write(
        agents.join(file),
        format!(
            "---\nname: {name}\ndescription: {name} description\ncolor: blue\n\
             skills: []\ntools: [Read]\n---\n"
        ),
    )
    .expect("agent fixture");
}

#[test]
fn filtered_agent_list_preserves_complete_index() {
    let temp = tempfile::tempdir().expect("tempdir");
    write_agent(temp.path(), "alpha.md", "alpha");
    write_agent(temp.path(), "beta.md", "beta");

    let initial = Command::new(godmode_bin())
        .args(["agent", "index"])
        .current_dir(temp.path())
        .output()
        .expect("generate index");
    assert!(initial.status.success());
    let expected =
        std::fs::read_to_string(temp.path().join("agents/INDEX.md")).expect("generated index");

    let filtered = Command::new(godmode_bin())
        .args(["agent", "list", "--filter", "alpha"])
        .current_dir(temp.path())
        .output()
        .expect("filtered list");
    assert!(filtered.status.success());

    let actual =
        std::fs::read_to_string(temp.path().join("agents/INDEX.md")).expect("preserved index");
    assert_eq!(actual, expected);
    assert!(actual.contains("| alpha |"));
    assert!(actual.contains("| beta |"));
}

#[test]
fn agent_index_check_rejects_duplicate_logical_names() {
    let temp = tempfile::tempdir().expect("tempdir");
    let cfg = temp.path().join("agents/cfg");
    std::fs::create_dir_all(&cfg).expect("cfg directory");
    let definition = "name: gm-duplicate\ndescription: Duplicate logical agent description\nmodel: inherit\ncolor: blue\ntools: [Read]\nskills: []\n";
    std::fs::write(cfg.join("alpha.cfg.yaml"), definition).expect("alpha cfg");
    std::fs::write(cfg.join("beta.cfg.yaml"), definition).expect("beta cfg");

    let output = Command::new(godmode_bin())
        .args(["agent", "index", "--check"])
        .current_dir(temp.path())
        .output()
        .expect("check index");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("duplicate"));
}
