/// End-to-end tests for `godmode visualize-graph`.
use std::process::Command;

fn godmode_bin() -> std::path::PathBuf {
    if let Ok(p) = std::env::var("CARGO_BIN_EXE_godmode") {
        return std::path::PathBuf::from(p);
    }
    let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest.parent().unwrap().parent().unwrap();
    workspace_root.join("target/debug/godmode")
}

fn setup_graph(yaml: &str) -> tempfile::TempDir {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let ctx = dir.path().join(".ctx").join("godmode");
    std::fs::create_dir_all(&ctx).unwrap();
    std::fs::write(ctx.join("tasks.yaml"), yaml).unwrap();
    dir
}

const CHAIN_YAML: &str = r#"tasks:
  - id: t1
    title: First task
    status: pending
  - id: t2
    title: Second task
    status: pending
    depends_on:
      - t1
"#;

#[test]
fn visualize_graph_dot_output_contains_nodes_and_edges() {
    let dir = setup_graph(CHAIN_YAML);
    let out = Command::new(godmode_bin())
        .args(["visualize-graph", "--format", "dot"])
        .current_dir(dir.path())
        .output()
        .expect("godmode binary must be present");

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "exit non-zero: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(stdout.contains("t1"), "DOT missing t1:\n{stdout}");
    assert!(stdout.contains("t2"), "DOT missing t2:\n{stdout}");
    assert!(stdout.contains("->"), "DOT missing edge arrow:\n{stdout}");
    // Should be valid DOT: starts with "digraph"
    assert!(
        stdout.contains("digraph"),
        "DOT missing digraph keyword:\n{stdout}"
    );
}

#[test]
fn visualize_graph_default_format_is_dot() {
    let dir = setup_graph(CHAIN_YAML);
    let out = Command::new(godmode_bin())
        .arg("visualize-graph")
        .current_dir(dir.path())
        .output()
        .expect("godmode binary must be present");

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "exit non-zero: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(stdout.contains("digraph"), "expected DOT output:\n{stdout}");
}

#[test]
fn visualize_graph_out_writes_file() {
    let dir = setup_graph(CHAIN_YAML);
    let out_path = dir.path().join("graph.dot");
    let out = Command::new(godmode_bin())
        .args([
            "visualize-graph",
            "--format",
            "dot",
            "--out",
            out_path.to_str().unwrap(),
        ])
        .current_dir(dir.path())
        .output()
        .expect("godmode binary must be present");

    assert!(
        out.status.success(),
        "exit non-zero: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(out_path.exists(), "output file not created");
    let content = std::fs::read_to_string(&out_path).unwrap();
    assert!(content.contains("t1"), "file missing t1:\n{content}");
    assert!(
        content.contains("digraph"),
        "file missing digraph:\n{content}"
    );
}
