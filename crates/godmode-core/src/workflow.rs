//! Causal workflow system — command-driven DAGs per agent.
//!
//! A workflow is a YAML file describing steps with `run:` commands and `depends_on` edges.
//! Steps execute in dependency order; each step must exit 0 before successors unlock.

use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::integrations::rx::resolve_cmd;

// ---------------------------------------------------------------------------
// Data model
// ---------------------------------------------------------------------------

/// Deserialized definition of a named causal workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDef {
    /// Workflow name used for persistence and display.
    pub name: String,
    #[serde(default)]
    /// Human-readable workflow description.
    pub description: String,
    #[serde(default)]
    /// Agent associated with the workflow.
    pub agent: String,
    /// Ordered set of steps in the workflow DAG.
    pub steps: Vec<WorkflowStep>,
}

/// One executable step and its dependency and branch edges.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    /// Unique step identifier within the workflow.
    pub id: String,
    /// Command executed for this step.
    pub run: String,
    #[serde(default)]
    /// Step IDs that must finish successfully first.
    pub depends_on: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Optional step to retain after successful completion.
    pub on_success: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Optional recovery step to retain after failure.
    pub on_failure: Option<String>,
}

/// Runtime lifecycle state of a workflow step.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StepState {
    /// Step is waiting for its dependencies.
    Pending,
    /// Step command is currently executing.
    Running,
    /// Step command completed successfully.
    Done,
    /// Step command completed unsuccessfully.
    Failed,
    /// Step was bypassed by a workflow branch.
    Skipped,
}

/// Persisted execution state for a workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowState {
    /// Name of the workflow being executed.
    pub workflow: String,
    /// Runtime status of each workflow step.
    pub steps: Vec<StepStatus>,
}

/// Runtime status and exit code for one workflow step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepStatus {
    /// Workflow step identifier.
    pub id: String,
    /// Current lifecycle state.
    pub state: StepState,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Process exit code after execution, when available.
    pub exit_code: Option<i32>,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Parse a workflow YAML file.
pub fn load(path: &Path) -> Result<WorkflowDef> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("reading workflow file {}", path.display()))?;
    serde_yaml::from_str(&raw).with_context(|| format!("parsing workflow YAML {}", path.display()))
}

/// Return steps whose dependencies are all Done and whose own state is Pending.
pub fn runnable_steps<'a>(def: &'a WorkflowDef, state: &WorkflowState) -> Vec<&'a WorkflowStep> {
    def.steps
        .iter()
        .filter(|step| {
            let status = state.steps.iter().find(|s| s.id == step.id);
            let is_pending = status
                .map(|s| s.state == StepState::Pending)
                .unwrap_or(true);
            if !is_pending {
                return false;
            }
            step.depends_on.iter().all(|dep_id| {
                state
                    .steps
                    .iter()
                    .find(|s| s.id == *dep_id)
                    .map(|s| s.state == StepState::Done)
                    .unwrap_or(false)
            })
        })
        .collect()
}

/// Run a workflow to completion. Executes steps in dependency order.
/// Returns the final `WorkflowState`.
pub fn run(def: &WorkflowDef, root: &Path) -> Result<WorkflowState> {
    let mut state = init_state(def);
    let state_path = root
        .join(".ctx")
        .join("godmode")
        .join(format!("workflow-{}.json", def.name));

    loop {
        let runnable: Vec<String> = runnable_steps(def, &state)
            .into_iter()
            .map(|s| s.id.clone())
            .collect();

        if runnable.is_empty() {
            break;
        }

        for step_id in runnable {
            let step = def.steps.iter().find(|s| s.id == step_id).unwrap();
            set_state(&mut state, &step_id, StepState::Running, None);

            let (prog, args) = resolve_cmd(&step.run);
            let exit_status = std::process::Command::new(&prog)
                .args(&args)
                .status()
                .with_context(|| format!("failed to launch step '{}': {}", step_id, step.run))?;

            let code = exit_status.code().unwrap_or(-1);

            if exit_status.success() {
                set_state(&mut state, &step_id, StepState::Done, Some(code));
                // follow on_success jump if set
                if let Some(ref target) = step.on_success.clone() {
                    skip_all_except(&mut state, def, target);
                }
            } else {
                set_state(&mut state, &step_id, StepState::Failed, Some(code));
                if let Some(ref target) = step.on_failure.clone() {
                    skip_all_except(&mut state, def, target);
                } else {
                    // no recovery — persist and stop
                    persist_state(&state, &state_path);
                    return Ok(state);
                }
            }

            persist_state(&state, &state_path);
        }
    }

    Ok(state)
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

fn init_state(def: &WorkflowDef) -> WorkflowState {
    WorkflowState {
        workflow: def.name.clone(),
        steps: def
            .steps
            .iter()
            .map(|s| StepStatus {
                id: s.id.clone(),
                state: StepState::Pending,
                exit_code: None,
            })
            .collect(),
    }
}

fn set_state(state: &mut WorkflowState, id: &str, new_state: StepState, code: Option<i32>) {
    if let Some(s) = state.steps.iter_mut().find(|s| s.id == id) {
        s.state = new_state;
        s.exit_code = code;
    }
}

/// Mark every Pending step as Skipped except the one with `keep_id`.
fn skip_all_except(state: &mut WorkflowState, def: &WorkflowDef, keep_id: &str) {
    // Find all step IDs that are NOT reachable from keep_id (via depends_on chains).
    // Simple approach: mark all Pending as Skipped, then re-Pending the keep target.
    for s in state.steps.iter_mut() {
        if s.state == StepState::Pending {
            s.state = StepState::Skipped;
        }
    }
    // Re-pending keep_id and all of its transitive dependents within the def.
    let reachable = reachable_from(def, keep_id);
    for s in state.steps.iter_mut() {
        if reachable.contains(&s.id) && s.state == StepState::Skipped {
            s.state = StepState::Pending;
        }
    }
}

fn reachable_from(def: &WorkflowDef, start: &str) -> Vec<String> {
    // BFS forward through steps that depend on `start` (and transitively).
    let mut visited: Vec<String> = vec![start.to_string()];
    let mut queue: Vec<String> = vec![start.to_string()];
    while !queue.is_empty() {
        let cur = queue.remove(0);
        for step in &def.steps {
            if step.depends_on.iter().any(|d| d == &cur) && !visited.contains(&step.id) {
                visited.push(step.id.clone());
                queue.push(step.id.clone());
            }
        }
    }
    visited
}

fn persist_state(state: &WorkflowState, path: &Path) {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(state) {
        let _ = std::fs::write(path, json);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_step(id: &str, run: &str, deps: Vec<&str>) -> WorkflowStep {
        WorkflowStep {
            id: id.to_string(),
            run: run.to_string(),
            depends_on: deps.into_iter().map(str::to_string).collect(),
            on_success: None,
            on_failure: None,
        }
    }

    fn two_step_def() -> WorkflowDef {
        WorkflowDef {
            name: "test-wf".to_string(),
            description: "a test workflow".to_string(),
            agent: "test-agent".to_string(),
            steps: vec![
                make_step("step1", "true", vec![]),
                make_step("step2", "true", vec!["step1"]),
            ],
        }
    }

    #[test]
    fn load_parses_yaml() {
        let tmp = std::env::temp_dir().join("godmode-workflow-test-load.yaml");
        std::fs::write(
            &tmp,
            "name: ci\ndescription: CI\nagent: tdd\nsteps:\n  - id: build\n    run: cargo build\n  - id: test\n    run: cargo test\n    depends_on: [build]\n",
        )
        .unwrap();
        let def = load(&tmp).unwrap();
        let _ = std::fs::remove_file(&tmp);
        assert_eq!(def.name, "ci");
        assert_eq!(def.steps.len(), 2);
        assert_eq!(def.steps[1].depends_on, vec!["build"]);
    }

    #[test]
    fn runnable_steps_returns_root_steps_initially() {
        let def = two_step_def();
        let state = init_state(&def);
        let runnable = runnable_steps(&def, &state);
        assert_eq!(runnable.len(), 1);
        assert_eq!(runnable[0].id, "step1");
    }

    #[test]
    fn runnable_steps_unlocks_after_dep_done() {
        let def = two_step_def();
        let mut state = init_state(&def);
        set_state(&mut state, "step1", StepState::Done, Some(0));
        let runnable = runnable_steps(&def, &state);
        assert_eq!(runnable.len(), 1);
        assert_eq!(runnable[0].id, "step2");
    }

    #[test]
    fn runnable_steps_empty_when_all_done() {
        let def = two_step_def();
        let mut state = init_state(&def);
        set_state(&mut state, "step1", StepState::Done, Some(0));
        set_state(&mut state, "step2", StepState::Done, Some(0));
        assert!(runnable_steps(&def, &state).is_empty());
    }

    #[test]
    fn run_executes_two_step_workflow() {
        let dir = std::env::temp_dir().join("godmode-workflow-run-test");
        std::fs::create_dir_all(dir.join(".ctx").join("godmode")).unwrap();
        let def = two_step_def();
        let final_state = run(&def, &dir).unwrap();
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(final_state.workflow, "test-wf");
        assert!(final_state.steps.iter().all(|s| s.state == StepState::Done));
    }

    #[test]
    fn run_marks_failed_on_non_zero_exit() {
        let dir = std::env::temp_dir().join("godmode-workflow-fail-test");
        std::fs::create_dir_all(dir.join(".ctx").join("godmode")).unwrap();
        let def = WorkflowDef {
            name: "fail-wf".to_string(),
            description: String::new(),
            agent: String::new(),
            steps: vec![make_step("bad", "false", vec![])],
        };
        let final_state = run(&def, &dir).unwrap();
        let _ = std::fs::remove_dir_all(&dir);
        let bad = final_state.steps.iter().find(|s| s.id == "bad").unwrap();
        assert_eq!(bad.state, StepState::Failed);
    }
}
