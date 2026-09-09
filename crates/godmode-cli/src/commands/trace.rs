use crate::TraceAction;
use anyhow::{Context, Result};
use godmode_core::trace_stats;
use std::path::Path;

pub fn run_trace_action(root: &Path, json: bool, action: TraceAction) -> Result<()> {
    match action {
        TraceAction::Tail {
            n,
            session,
            current,
        } => {
            anyhow::ensure!(n <= 1000, "--n must be between 0 and 1000");
            let session = resolve(root, session, current)?;
            let events =
                trace_stats::tail(root, n, session.as_deref()).context("querying trace tail")?;
            if json {
                println!("{}", serde_json::to_string_pretty(&events)?)
            } else if events.is_empty() {
                println!("No trace events.")
            } else {
                for event in events {
                    println!("{}", serde_json::to_string(&event)?);
                }
            }
        }
        TraceAction::Failures { session, current } => {
            let session = resolve(root, session, current)?;
            let events = trace_stats::failures(root, session.as_deref())
                .context("querying trace failures")?;
            if json {
                println!("{}", serde_json::to_string_pretty(&events)?)
            } else if events.is_empty() {
                println!("No failures.")
            } else {
                for event in events {
                    println!("{}", serde_json::to_string(&event)?);
                }
            }
        }
        TraceAction::Stats { session, current } => {
            let session = resolve(root, session, current)?;
            let report = trace_stats::stats(root, session.as_deref())
                .context("querying trace statistics")?;
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?)
            } else {
                print_stats(&report);
            }
        }
        TraceAction::Summary { sessions, previous } => {
            anyhow::ensure!(sessions > 0, "--sessions must be greater than zero");
            let current = if previous {
                trace_stats::current_session_id(root).context("resolving current trace session")?
            } else {
                None
            };
            let summaries = trace_stats::summaries(root, sessions, current.as_deref())
                .context("querying trace summaries")?;
            if json {
                println!("{}", serde_json::to_string_pretty(&summaries)?)
            } else if summaries.is_empty() {
                println!("No traced sessions.")
            } else {
                for summary in summaries {
                    println!(
                        "--- {} @ {}\n    errors={} blocked={} denied={} agents: {} complete / {} running / {} decisions",
                        summary.session_id,
                        summary.started_at,
                        summary.errors,
                        summary.agents_blocked,
                        summary.agents_denied,
                        summary.agents_complete,
                        summary.agents_running,
                        summary.decisions
                    );
                }
            }
        }
    }
    Ok(())
}
fn resolve(root: &Path, session: Option<String>, current: bool) -> Result<Option<String>> {
    if current {
        return trace_stats::current_session_id(root)
            .context("resolving current trace session")?
            .map(Some)
            .ok_or_else(|| anyhow::anyhow!("no current traced session"));
    }
    Ok(session)
}
fn print_stats(report: &trace_stats::TraceStats) {
    println!("=== skill durations (ms) ===");
    if report.skills.is_empty() {
        println!("(none)")
    } else {
        println!("skill\truns\tavg_ms\tmax_ms");
        for skill in &report.skills {
            println!(
                "{}\t{}\t{}\t{}",
                skill.skill, skill.runs, skill.avg_ms, skill.max_ms
            );
        }
    }
    println!("\n=== agent convergence ===");
    if report.agents.is_empty() {
        println!("(no agents)")
    } else {
        for agent in &report.agents {
            println!("{}: {}", agent.agent_id, agent.status);
        }
    }
    println!("\n=== decisions ===");
    if report.decisions.is_empty() {
        println!("(none)")
    } else {
        for decision in &report.decisions {
            println!("{}", serde_json::to_string(decision).unwrap_or_default());
        }
    }
    if !report.failures.is_empty() {
        println!("\nFailures: {}", report.failures.len());
        for failure in &report.failures {
            println!("{}", serde_json::to_string(failure).unwrap_or_default());
        }
    }
    if report.legacy_rows > 0 || report.malformed_rows > 0 {
        println!(
            "\nIgnored rows: {} legacy, {} malformed",
            report.legacy_rows, report.malformed_rows
        );
    }
}
