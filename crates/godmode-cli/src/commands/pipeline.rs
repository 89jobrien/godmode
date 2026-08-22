//! Named multi-step skill pipelines.

use anyhow::Result;
use godmode_core::pipeline;
use std::path::Path;

use crate::PipelineAction;

pub fn run_pipeline_action(root: &Path, json: bool, action: PipelineAction) -> Result<()> {
    match action {
        PipelineAction::List => {
            let pipelines = pipeline::load_pipelines(root)?;
            if pipelines.is_empty() {
                if json {
                    println!("[]");
                } else {
                    println!("No pipelines found.");
                }
                return Ok(());
            }
            if json {
                println!("{}", serde_json::to_string_pretty(&pipelines)?);
            } else {
                for p in &pipelines {
                    println!("{} — {}", p.name, p.description);
                }
            }
            Ok(())
        }

        PipelineAction::Show { name } => {
            let p = pipeline::load_pipeline(root, &name)?;
            let state = pipeline::load_state(root)?;
            let active_idx = state
                .as_ref()
                .filter(|s| s.active == name)
                .map(|s| s.current_step);
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "pipeline": p,
                        "current_step": active_idx,
                    }))?
                );
            } else {
                println!("Pipeline: {} — {}", p.name, p.description);
                for (i, step) in p.steps.iter().enumerate() {
                    let marker = if active_idx == Some(i) { ">>" } else { "  " };
                    println!("{} [{}] {}", marker, i + 1, step.skill);
                }
            }
            Ok(())
        }

        PipelineAction::Start { name, from } => {
            let p = pipeline::load_pipeline(root, &name)?;
            let state = pipeline::start(&p, from.as_deref())?;
            let first = pipeline::current_step(&state, &p)
                .map(|s| s.skill.as_str())
                .unwrap_or("(none)");
            pipeline::save_state(root, &state)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&state)?);
            } else {
                println!("Pipeline '{}' started at step: {}", name, first);
            }
            Ok(())
        }

        PipelineAction::Next => advance_pipeline(root, json, pipeline::advance),

        PipelineAction::Skip => advance_pipeline(root, json, pipeline::skip),

        PipelineAction::Stop => {
            pipeline::clear_state(root)?;
            if json {
                println!("{}", serde_json::json!({"ok": true}));
            } else {
                println!("Pipeline stopped.");
            }
            Ok(())
        }

        PipelineAction::Run {
            name,
            from: _from,
            fail_fast,
        } => {
            let result = pipeline::run_tasks(root, &name, fail_fast)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                for sr in &result.steps {
                    if sr.skipped {
                        println!("  [skip] {}", sr.skill);
                    } else {
                        println!(
                            "  [{}] {} — {} task(s), {} failed",
                            if sr.tasks_failed > 0 { "FAIL" } else { "ok" },
                            sr.skill,
                            sr.tasks_run,
                            sr.tasks_failed,
                        );
                    }
                }
                if result.completed {
                    println!("Pipeline complete.");
                } else if let Some(ref skill) = result.stopped_at {
                    println!("Stopped at: {skill}");
                    std::process::exit(1);
                }
            }
            Ok(())
        }

        PipelineAction::Status => {
            let state = pipeline::load_state(root)?;
            match state {
                None => {
                    if json {
                        println!("null");
                    } else {
                        println!("No active pipeline.");
                    }
                }
                Some(s) => {
                    let p = pipeline::load_pipeline(root, &s.active)?;
                    let (done, total) = pipeline::progress(&s, &p);
                    let current = pipeline::current_step(&s, &p)
                        .map(|step| step.skill.as_str())
                        .unwrap_or("(complete)");
                    if json {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "active": s.active,
                                "current_step": current,
                                "progress": { "done": done, "total": total },
                                "complete": pipeline::is_complete(&s, &p),
                            }))?
                        );
                    } else {
                        println!("Pipeline: {}", s.active);
                        println!("Step:     {}", current);
                        println!("Progress: {}/{}", done, total);
                    }
                }
            }
            Ok(())
        }
    }
}

fn advance_pipeline(
    root: &std::path::Path,
    json: bool,
    op: for<'a> fn(
        &mut pipeline::PipelineState,
        &'a pipeline::Pipeline,
    ) -> Option<&'a pipeline::PipelineStep>,
) -> Result<()> {
    let mut state =
        pipeline::load_state(root)?.ok_or_else(|| anyhow::anyhow!("No active pipeline."))?;
    let p = pipeline::load_pipeline(root, &state.active.clone())?;
    let next = op(&mut state, &p);
    pipeline::save_state(root, &state)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&state)?);
    } else if let Some(step) = next {
        println!("Advanced to: {}", step.skill);
    } else {
        println!("Pipeline complete.");
    }
    Ok(())
}
