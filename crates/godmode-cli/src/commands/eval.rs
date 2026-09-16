use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::*;

pub fn handle(command: Cmd, root: &Path, json: bool, _sarif: bool) -> Result<()> {
    let Cmd::Eval { action } = command else {
        unreachable!("dispatcher sent command to the wrong handler")
    };
    match action {
        EvalAction::Run {
            skill,
            results,
            repeat,
            max_cases,
            max_cost_usd,
        } => {
            let mut executor = eval::FixtureExecutor::load(&PathBuf::from(results))?;
            let report = eval::run(
                root,
                &skill,
                eval::RunPolicy {
                    repetitions: repeat,
                    max_cases,
                    max_cost_usd,
                },
                &mut executor,
            )?;
            print_run(&report, json)
        }
        EvalAction::Compare { skill } => {
            let comparison = eval::compare(root, &skill)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&comparison)?);
            } else {
                println!("Skill:           {}", comparison.skill);
                println!(
                    "Pass-rate delta: {:+.1}%",
                    comparison.pass_rate_delta * 100.0
                );
                println!("Cost delta:      {:+.4} USD", comparison.cost_delta_usd);
            }
            Ok(())
        }
        EvalAction::Promote {
            skill,
            min_pass_rate,
            max_cost_usd,
        } => {
            let report = eval::promote(
                root,
                &skill,
                eval::Gate {
                    min_pass_rate,
                    max_cost_usd,
                },
            )?;
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!(
                    "Promoted evaluation run {} for {}.",
                    report.id, report.skill
                );
            }
            Ok(())
        }
        EvalAction::Status {
            skill,
            min_pass_rate,
            max_cost_usd,
        } => {
            let status = eval::status(
                root,
                &skill,
                eval::Gate {
                    min_pass_rate,
                    max_cost_usd,
                },
            )?;
            if json {
                println!("{}", serde_json::to_string_pretty(&status)?);
            } else {
                println!("Skill:       {}", status.skill);
                println!("Stored runs: {}", status.stored_runs);
                match &status.latest {
                    Some(latest) => {
                        println!("Pass rate:   {:.1}%", latest.pass_rate * 100.0);
                        println!("Cost:        ${:.4}", latest.cost_usd);
                    }
                    None => println!("Latest:      none"),
                }
                println!(
                    "Baseline:    {}",
                    status
                        .baseline
                        .as_ref()
                        .map_or("none", |baseline| baseline.id.as_str())
                );
                println!(
                    "Gate:        {}",
                    if status.gate_passed { "PASS" } else { "FAIL" }
                );
            }
            if !status.gate_passed {
                anyhow::bail!("evaluation gate failed for skill '{skill}'");
            }
            Ok(())
        }
    }
}

fn print_run(report: &eval::RunReport, json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(report)?);
    } else {
        println!("Evaluation run: {}", report.id);
        println!("Skill:          {}", report.skill);
        println!("Cases:          {}/{} passed", report.passed, report.total);
        println!("Pass rate:      {:.1}%", report.pass_rate * 100.0);
        println!("Cost:           ${:.4}", report.cost_usd);
    }
    Ok(())
}
