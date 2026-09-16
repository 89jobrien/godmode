use std::process::{Command, Output};

use godmode_core::pipeline::{self, Pipeline, PipelineStep};

fn godmode_bin() -> std::path::PathBuf {
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_godmode") {
        return path.into();
    }
    let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("target/debug/godmode")
}

fn setup_pipeline() -> (tempfile::TempDir, Pipeline) {
    let dir = tempfile::tempdir().unwrap();
    let pipeline = Pipeline {
        name: "entry-points".into(),
        description: "test pipeline entry points".into(),
        steps: vec![
            PipelineStep {
                skill: "first".into(),
                optional: false,
                r#loop: None,
                parallel_with: vec![],
            },
            PipelineStep {
                skill: "second".into(),
                optional: false,
                r#loop: None,
                parallel_with: vec![],
            },
        ],
        entry_points: vec!["first".into(), "second".into()],
    };
    let pipelines_dir = dir.path().join("pipelines");
    std::fs::create_dir_all(&pipelines_dir).unwrap();
    std::fs::write(
        pipelines_dir.join("entry-points.yaml"),
        "name: entry-points\ndescription: test pipeline entry points\nsteps:\n  - skill: first\n  - skill: second\nentry_points:\n  - first\n  - second\n",
    )
    .unwrap();
    (dir, pipeline)
}

fn run_pipeline(root: &std::path::Path, from: Option<&str>) -> Output {
    let mut command = Command::new(godmode_bin());
    command.args(["--json", "pipeline", "run", "entry-points"]);
    if let Some(from) = from {
        command.args(["--from", from]);
    }
    command.current_dir(root).output().unwrap()
}

fn step_skills(output: &Output) -> Vec<String> {
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    value["steps"]
        .as_array()
        .unwrap()
        .iter()
        .map(|step| step["skill"].as_str().unwrap().to_owned())
        .collect()
}

#[test]
fn fresh_run_with_valid_from_starts_at_named_entry_point() {
    let (dir, _) = setup_pipeline();

    let output = run_pipeline(dir.path(), Some("second"));

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(step_skills(&output), ["second"]);
}

#[test]
fn fresh_run_with_invalid_from_returns_validation_error() {
    let (dir, _) = setup_pipeline();

    let output = run_pipeline(dir.path(), Some("missing"));

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("skill 'missing' is not a step in pipeline 'entry-points'")
    );
}

#[test]
fn fresh_run_without_from_starts_at_first_step() {
    let (dir, _) = setup_pipeline();

    let output = run_pipeline(dir.path(), None);

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(step_skills(&output), ["first", "second"]);
}

#[test]
fn matching_active_state_ignores_supplied_from() {
    let (dir, definition) = setup_pipeline();
    let mut state = pipeline::start(&definition, None).unwrap();
    pipeline::advance(&mut state, &definition);
    pipeline::save_state(dir.path(), &state).unwrap();

    let output = run_pipeline(dir.path(), Some("missing"));

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(step_skills(&output), ["second"]);
}
