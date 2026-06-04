use anyhow::{Context, Result, bail};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A single step in a pipeline definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStep {
    pub skill: String,
    #[serde(default)]
    pub optional: bool,
    /// `per-task` repeats the skill for each runnable task in the graph.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#loop: Option<LoopMode>,
    /// Run these skills in parallel alongside this step.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parallel_with: Vec<String>,
}

/// How a pipeline step repeats.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LoopMode {
    PerTask,
}

/// A named pipeline definition loaded from `pipelines/<name>.yaml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pipeline {
    pub name: String,
    pub description: String,
    pub steps: Vec<PipelineStep>,
    #[serde(default)]
    pub entry_points: Vec<String>,
}

/// Status of a single step in the execution history.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StepStatus {
    Pending,
    Running,
    Done,
    Skipped,
}

impl std::fmt::Display for StepStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StepStatus::Pending => write!(f, "pending"),
            StepStatus::Running => write!(f, "running"),
            StepStatus::Done => write!(f, "done"),
            StepStatus::Skipped => write!(f, "skipped"),
        }
    }
}

/// A completed or in-progress step in the pipeline history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepRecord {
    pub skill: String,
    pub status: StepStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,
}

/// Persisted pipeline execution state at `.ctx/godmode/pipeline.yaml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineState {
    pub active: String,
    pub current_step: usize,
    pub started_at: DateTime<Utc>,
    #[serde(default)]
    pub history: Vec<StepRecord>,
}

// ---------------------------------------------------------------------------
// File paths
// ---------------------------------------------------------------------------

/// Directory containing pipeline definitions.
pub fn pipelines_dir(root: &Path) -> PathBuf {
    root.join("pipelines")
}

/// Path to the active pipeline state file.
pub fn state_file(root: &Path) -> PathBuf {
    root.join(".ctx").join("godmode").join("pipeline.yaml")
}

// ---------------------------------------------------------------------------
// Load / save
// ---------------------------------------------------------------------------

/// Load all pipeline definitions from `pipelines/*.yaml`.
pub fn load_pipelines(root: &Path) -> Result<Vec<Pipeline>> {
    let dir = pipelines_dir(root);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut pipelines = Vec::new();
    let mut entries: Vec<_> = std::fs::read_dir(&dir)
        .with_context(|| format!("reading {}", dir.display()))?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .is_some_and(|ext| ext == "yaml" || ext == "yml")
        })
        .collect();
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let path = entry.path();
        let raw = std::fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        let pipeline: Pipeline =
            serde_yaml::from_str(&raw).with_context(|| format!("parsing {}", path.display()))?;
        pipelines.push(pipeline);
    }
    Ok(pipelines)
}

/// Load a single pipeline by name.
pub fn load_pipeline(root: &Path, name: &str) -> Result<Pipeline> {
    let path = pipelines_dir(root).join(format!("{name}.yaml"));
    if !path.exists() {
        bail!("pipeline '{name}' not found at {}", path.display());
    }
    let raw =
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    serde_yaml::from_str(&raw).with_context(|| format!("parsing {}", path.display()))
}

/// Load the active pipeline state. Returns `None` if no pipeline is active.
pub fn load_state(root: &Path) -> Result<Option<PipelineState>> {
    let path = state_file(root);
    if !path.exists() {
        return Ok(None);
    }
    let raw =
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    let state: PipelineState =
        serde_yaml::from_str(&raw).with_context(|| format!("parsing {}", path.display()))?;
    Ok(Some(state))
}

/// Persist pipeline state to disk.
pub fn save_state(root: &Path, state: &PipelineState) -> Result<()> {
    let path = state_file(root);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let raw = serde_yaml::to_string(state)?;
    std::fs::write(&path, raw).with_context(|| format!("writing {}", path.display()))
}

/// Remove the pipeline state file (deactivate).
pub fn clear_state(root: &Path) -> Result<()> {
    let path = state_file(root);
    if path.exists() {
        std::fs::remove_file(&path).with_context(|| format!("removing {}", path.display()))?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// State machine
// ---------------------------------------------------------------------------

/// Initialize a new pipeline run. Validates the pipeline name and optional
/// `--from` entry point. Returns the initial state with `current_step` set
/// to the starting position.
pub fn start(pipeline: &Pipeline, from: Option<&str>) -> Result<PipelineState> {
    let start_idx = match from {
        Some(skill) => {
            let idx = pipeline
                .steps
                .iter()
                .position(|s| s.skill == skill)
                .with_context(|| {
                    format!(
                        "skill '{skill}' is not a step in pipeline '{}'",
                        pipeline.name
                    )
                })?;
            // Validate it's a valid entry point if entry_points is non-empty.
            if !pipeline.entry_points.is_empty()
                && !pipeline.entry_points.iter().any(|e| e == skill)
            {
                bail!(
                    "skill '{skill}' is not a valid entry point for pipeline '{}'; \
                     valid entry points: {}",
                    pipeline.name,
                    pipeline.entry_points.join(", ")
                );
            }
            idx
        }
        None => 0,
    };

    Ok(PipelineState {
        active: pipeline.name.clone(),
        current_step: start_idx,
        started_at: Utc::now(),
        history: Vec::new(),
    })
}

/// Return the current step to execute, or `None` if the pipeline is complete.
pub fn current_step<'a>(state: &PipelineState, pipeline: &'a Pipeline) -> Option<&'a PipelineStep> {
    pipeline.steps.get(state.current_step)
}

/// Mark the current step as done and advance to the next step.
/// Returns the next step to execute, or `None` if the pipeline is complete.
pub fn advance<'a>(state: &mut PipelineState, pipeline: &'a Pipeline) -> Option<&'a PipelineStep> {
    // Record completion of the current step.
    if let Some(step) = pipeline.steps.get(state.current_step) {
        state.history.push(StepRecord {
            skill: step.skill.clone(),
            status: StepStatus::Done,
            completed_at: Some(Utc::now()),
        });
    }
    state.current_step += 1;
    pipeline.steps.get(state.current_step)
}

/// Skip the current step without executing and advance.
/// Returns the next step, or `None` if the pipeline is complete.
pub fn skip<'a>(state: &mut PipelineState, pipeline: &'a Pipeline) -> Option<&'a PipelineStep> {
    if let Some(step) = pipeline.steps.get(state.current_step) {
        state.history.push(StepRecord {
            skill: step.skill.clone(),
            status: StepStatus::Skipped,
            completed_at: Some(Utc::now()),
        });
    }
    state.current_step += 1;
    pipeline.steps.get(state.current_step)
}

/// Returns `true` when `current_step` is past the last step.
pub fn is_complete(state: &PipelineState, pipeline: &Pipeline) -> bool {
    state.current_step >= pipeline.steps.len()
}

/// Remaining step count (including current).
pub fn remaining(state: &PipelineState, pipeline: &Pipeline) -> usize {
    pipeline.steps.len().saturating_sub(state.current_step)
}

/// Progress as (completed, total).
pub fn progress(state: &PipelineState, pipeline: &Pipeline) -> (usize, usize) {
    (state.current_step, pipeline.steps.len())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_pipeline() -> Pipeline {
        Pipeline {
            name: "test".into(),
            description: "test pipeline".into(),
            steps: vec![
                PipelineStep {
                    skill: "brainstorm".into(),
                    optional: true,
                    r#loop: None,
                    parallel_with: vec![],
                },
                PipelineStep {
                    skill: "context-map".into(),
                    optional: false,
                    r#loop: None,
                    parallel_with: vec![],
                },
                PipelineStep {
                    skill: "writing-plans".into(),
                    optional: false,
                    r#loop: None,
                    parallel_with: vec![],
                },
                PipelineStep {
                    skill: "task-driven-development".into(),
                    optional: false,
                    r#loop: Some(LoopMode::PerTask),
                    parallel_with: vec![],
                },
            ],
            entry_points: vec!["brainstorm".into(), "context-map".into()],
        }
    }

    #[test]
    fn start_from_beginning() {
        let p = sample_pipeline();
        let state = start(&p, None).expect("start should succeed");
        assert_eq!(state.current_step, 0);
        assert_eq!(state.active, "test");
        assert!(state.history.is_empty());
    }

    #[test]
    fn start_from_entry_point() {
        let p = sample_pipeline();
        let state = start(&p, Some("context-map")).expect("start from entry point");
        assert_eq!(state.current_step, 1);
    }

    #[test]
    fn start_from_invalid_entry_point_fails() {
        let p = sample_pipeline();
        let err = start(&p, Some("writing-plans")).unwrap_err();
        assert!(
            err.to_string().contains("not a valid entry point"),
            "got: {err}"
        );
    }

    #[test]
    fn start_from_nonexistent_skill_fails() {
        let p = sample_pipeline();
        let err = start(&p, Some("nonexistent")).unwrap_err();
        assert!(
            err.to_string().contains("not a step in pipeline"),
            "got: {err}"
        );
    }

    #[test]
    fn advance_walks_through_steps() {
        let p = sample_pipeline();
        let mut state = start(&p, None).unwrap();

        let step = current_step(&state, &p).expect("should have current step");
        assert_eq!(step.skill, "brainstorm");

        let next = advance(&mut state, &p).expect("should have next");
        assert_eq!(next.skill, "context-map");
        assert_eq!(state.current_step, 1);
        assert_eq!(state.history.len(), 1);
        assert_eq!(state.history[0].status, StepStatus::Done);

        let next = advance(&mut state, &p).expect("should have next");
        assert_eq!(next.skill, "writing-plans");

        let next = advance(&mut state, &p).expect("should have next");
        assert_eq!(next.skill, "task-driven-development");
        assert!(
            next.r#loop
                .as_ref()
                .is_some_and(|l| *l == LoopMode::PerTask)
        );

        let done = advance(&mut state, &p);
        assert!(done.is_none(), "pipeline should be complete");
        assert!(is_complete(&state, &p));
    }

    #[test]
    fn skip_records_skipped_status() {
        let p = sample_pipeline();
        let mut state = start(&p, None).unwrap();

        let next = skip(&mut state, &p).expect("should have next after skip");
        assert_eq!(next.skill, "context-map");
        assert_eq!(state.history.len(), 1);
        assert_eq!(state.history[0].status, StepStatus::Skipped);
        assert_eq!(state.history[0].skill, "brainstorm");
    }

    #[test]
    fn progress_and_remaining() {
        let p = sample_pipeline();
        let mut state = start(&p, None).unwrap();

        assert_eq!(progress(&state, &p), (0, 4));
        assert_eq!(remaining(&state, &p), 4);

        advance(&mut state, &p);
        assert_eq!(progress(&state, &p), (1, 4));
        assert_eq!(remaining(&state, &p), 3);

        advance(&mut state, &p);
        advance(&mut state, &p);
        advance(&mut state, &p);
        assert_eq!(progress(&state, &p), (4, 4));
        assert_eq!(remaining(&state, &p), 0);
        assert!(is_complete(&state, &p));
    }

    #[test]
    fn pipeline_roundtrips_yaml() {
        let p = sample_pipeline();
        let yaml = serde_yaml::to_string(&p).expect("serialize");
        let back: Pipeline = serde_yaml::from_str(&yaml).expect("deserialize");
        assert_eq!(back.name, "test");
        assert_eq!(back.steps.len(), 4);
        assert!(back.steps[0].optional);
        assert_eq!(back.steps[3].r#loop, Some(LoopMode::PerTask));
        assert_eq!(back.entry_points, vec!["brainstorm", "context-map"]);
    }

    #[test]
    fn state_roundtrips_yaml() {
        let p = sample_pipeline();
        let mut state = start(&p, None).unwrap();
        advance(&mut state, &p);

        let yaml = serde_yaml::to_string(&state).expect("serialize");
        let back: PipelineState = serde_yaml::from_str(&yaml).expect("deserialize");
        assert_eq!(back.active, "test");
        assert_eq!(back.current_step, 1);
        assert_eq!(back.history.len(), 1);
        assert_eq!(back.history[0].status, StepStatus::Done);
    }

    #[test]
    fn parallel_with_roundtrips() {
        let step = PipelineStep {
            skill: "dead-code".into(),
            optional: false,
            r#loop: None,
            parallel_with: vec!["dep-audit".into(), "introspection".into()],
        };
        let yaml = serde_yaml::to_string(&step).expect("serialize");
        assert!(yaml.contains("parallel_with"), "got: {yaml}");
        let back: PipelineStep = serde_yaml::from_str(&yaml).expect("deserialize");
        assert_eq!(back.parallel_with.len(), 2);
    }

    #[test]
    fn empty_parallel_with_omitted_from_yaml() {
        let step = PipelineStep {
            skill: "brainstorm".into(),
            optional: false,
            r#loop: None,
            parallel_with: vec![],
        };
        let yaml = serde_yaml::to_string(&step).expect("serialize");
        assert!(
            !yaml.contains("parallel_with"),
            "empty parallel_with should be omitted: {yaml}"
        );
    }

    #[test]
    fn load_pipelines_empty_dir() {
        let dir = tempfile::tempdir().unwrap();
        let result = load_pipelines(dir.path()).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn load_and_save_state_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join(".ctx").join("godmode")).unwrap();

        let p = sample_pipeline();
        let mut state = start(&p, None).unwrap();
        advance(&mut state, &p);

        save_state(root, &state).unwrap();
        let loaded = load_state(root).unwrap().expect("state should exist");
        assert_eq!(loaded.active, "test");
        assert_eq!(loaded.current_step, 1);
        assert_eq!(loaded.history.len(), 1);
    }

    #[test]
    fn load_state_returns_none_when_missing() {
        let dir = tempfile::tempdir().unwrap();
        let result = load_state(dir.path()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn clear_state_removes_file() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join(".ctx").join("godmode")).unwrap();

        let p = sample_pipeline();
        let state = start(&p, None).unwrap();
        save_state(root, &state).unwrap();
        assert!(state_file(root).exists());

        clear_state(root).unwrap();
        assert!(!state_file(root).exists());
    }

    #[test]
    fn clear_state_noop_when_missing() {
        let dir = tempfile::tempdir().unwrap();
        clear_state(dir.path()).unwrap(); // should not error
    }

    #[test]
    fn load_pipeline_by_name() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let pdir = root.join("pipelines");
        std::fs::create_dir_all(&pdir).unwrap();

        let p = sample_pipeline();
        let yaml = serde_yaml::to_string(&p).unwrap();
        std::fs::write(pdir.join("test.yaml"), &yaml).unwrap();

        let loaded = load_pipeline(root, "test").unwrap();
        assert_eq!(loaded.name, "test");
        assert_eq!(loaded.steps.len(), 4);
    }

    #[test]
    fn load_pipeline_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let err = load_pipeline(dir.path(), "nope").unwrap_err();
        assert!(err.to_string().contains("not found"), "got: {err}");
    }
}
