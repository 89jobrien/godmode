//! Per-agent workflow DAG execution.

use anyhow::Result;
use godmode_core::{agent, workflow};
use std::path::Path;

use crate::WorkflowAction;

pub fn run_workflow_action(root: &Path, json: bool, action: WorkflowAction) -> Result<()> {
    match action {
        WorkflowAction::Run {
            agent: agent_name,
            workflow: wf_name,
        } => {
            let agents_dir = root.join("agents");
            let agent_path = agents_dir.join(format!("{}.yaml", agent_name));
            let agent_def = agent::load(&agent_path)?;
            let wf_ref = agent_def
                .workflows
                .iter()
                .find(|w| w.name == wf_name)
                .ok_or_else(|| {
                    anyhow::anyhow!("workflow '{}' not found in agent '{}'", wf_name, agent_name)
                })?;
            let wf_path = root.join(&wf_ref.path);
            let wf_def = workflow::load(&wf_path)?;
            let final_state = workflow::run(&wf_def, root)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&final_state)?);
            } else {
                for s in &final_state.steps {
                    let state_str = format!("{:?}", s.state);
                    let code = s
                        .exit_code
                        .map(|c| format!(" (exit {})", c))
                        .unwrap_or_default();
                    println!("[{:8}] {}{}", state_str, s.id, code);
                }
            }
            Ok(())
        }

        WorkflowAction::List {
            agent: agent_filter,
        } => {
            let agents_dir = root.join("agents");
            let mut entries: Vec<serde_json::Value> = vec![];
            if agents_dir.exists() {
                let yaml_files: Vec<std::path::PathBuf> = std::fs::read_dir(&agents_dir)?
                    .filter_map(|e| e.ok())
                    .map(|e| e.path())
                    .filter(|p| p.extension().and_then(|x| x.to_str()) == Some("yaml"))
                    .collect();
                for yf in yaml_files {
                    let Ok(a) = agent::load(&yf) else { continue };
                    if agent_filter.as_deref().is_some_and(|f| a.name != f) {
                        continue;
                    }
                    for wf in &a.workflows {
                        entries.push(serde_json::json!({
                            "agent": a.name,
                            "workflow": wf.name,
                            "path": wf.path,
                            "slash_command": wf.slash_command,
                        }));
                    }
                }
            }
            if json {
                println!("{}", serde_json::to_string_pretty(&entries)?);
            } else if entries.is_empty() {
                println!("No workflows found.");
            } else {
                println!("{:<30} {:<30} PATH", "AGENT", "WORKFLOW");
                for e in &entries {
                    println!(
                        "{:<30} {:<30} {}",
                        e["agent"].as_str().unwrap_or(""),
                        e["workflow"].as_str().unwrap_or(""),
                        e["path"].as_str().unwrap_or(""),
                    );
                }
            }
            Ok(())
        }

        WorkflowAction::Status { name } => {
            let state_path = root
                .join(".ctx")
                .join("godmode")
                .join(format!("workflow-{}.json", name));
            if !state_path.exists() {
                if json {
                    println!("null");
                } else {
                    println!("No state file found for workflow '{}'.", name);
                }
                return Ok(());
            }
            let raw = std::fs::read_to_string(&state_path)?;
            let state: workflow::WorkflowState = serde_json::from_str(&raw)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&state)?);
            } else {
                println!("Workflow: {}", state.workflow);
                for s in &state.steps {
                    let state_str = format!("{:?}", s.state);
                    let code = s
                        .exit_code
                        .map(|c| format!(" (exit {})", c))
                        .unwrap_or_default();
                    println!("[{:8}] {}{}", state_str, s.id, code);
                }
            }
            Ok(())
        }
    }
}
