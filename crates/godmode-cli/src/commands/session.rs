//! Session lifecycle and trace query adapters.

use std::path::Path;

use anyhow::Result;
use godmode_core::{integrations, trace_stats};

use crate::{SessionAction, TraceAction};

pub(super) fn run_handon(root: &Path, json: bool, compact: bool) -> Result<()> {
    let out = integrations::handon(root)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else if compact {
        let graph = &out.graph;
        println!(
            "godmode: {}D {}R {}P {}B",
            graph.done, graph.running, graph.pending, graph.blocked
        );
    } else {
        print!("{}", out.human);
    }
    Ok(())
}

pub(super) fn run_handoff(root: &Path, json: bool) -> Result<()> {
    let out = integrations::handoff(root)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        print!("{}", out.human);
    }
    Ok(())
}

pub(super) fn run_session(root: &Path, json: bool, action: SessionAction) -> Result<()> {
    match action {
        SessionAction::Prune {
            older_than,
            dry_run,
        } => {
            use godmode_core::integrations::crux;
            use godmode_core::session::prune_sessions_older_than;

            let sessions_dir = crux::sessions_dir(root);
            let pruned = prune_sessions_older_than(&sessions_dir, older_than, dry_run)?;
            if json {
                let paths: Vec<String> = pruned
                    .iter()
                    .map(|path| path.display().to_string())
                    .collect();
                println!("{}", serde_json::to_string_pretty(&paths)?);
            } else if pruned.is_empty() {
                println!("No session files to prune.");
            } else if dry_run {
                println!("{} file(s) would be deleted.", pruned.len());
            } else {
                println!("Pruned {} session file(s).", pruned.len());
            }
        }
    }
    Ok(())
}

pub(super) fn run_trace(root: &Path, json: bool, action: TraceAction) -> Result<()> {
    match action {
        TraceAction::Tail {
            n,
            session,
            current,
        } => {
            anyhow::ensure!((1..=1000).contains(&n), "--n must be between 1 and 1000");
            let session = resolve_trace_session(root, session, current)?;
            let events = trace_stats::tail(root, n, session.as_deref())?;
            render_trace_events(json, &events, "No trace events.")?;
        }
        TraceAction::Failures { session, current } => {
            let session = resolve_trace_session(root, session, current)?;
            let events = trace_stats::failures(root, session.as_deref())?;
            render_trace_events(json, &events, "No failures.")?;
        }
        TraceAction::Stats { session, current } => {
            let session = resolve_trace_session(root, session, current)?;
            let report = trace_stats::stats(root, session.as_deref())?;
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                print_trace_stats(&report);
            }
        }
        TraceAction::Summary { sessions, previous } => {
            anyhow::ensure!(sessions > 0, "--sessions must be greater than zero");
            let current_session = if previous {
                trace_stats::current_session_id(root)?
            } else {
                None
            };
            let summaries = trace_stats::summaries(root, sessions, current_session.as_deref())?;
            if json {
                println!("{}", serde_json::to_string_pretty(&summaries)?);
            } else if summaries.is_empty() {
                println!("No traced sessions.");
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

fn render_trace_events(json: bool, events: &[serde_json::Value], empty: &str) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(events)?);
    } else if events.is_empty() {
        println!("{empty}");
    } else {
        for event in events {
            println!("{}", serde_json::to_string(event)?);
        }
    }
    Ok(())
}

fn resolve_trace_session(
    root: &Path,
    session: Option<String>,
    current: bool,
) -> Result<Option<String>> {
    if current {
        return trace_stats::current_session_id(root)?
            .map(Some)
            .ok_or_else(|| anyhow::anyhow!("no current traced session"));
    }
    Ok(session)
}

fn print_trace_stats(report: &trace_stats::TraceStats) {
    println!("=== skill durations (ms) ===");
    if report.skills.is_empty() {
        println!("(none)");
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
        println!("(no agents)");
    } else {
        for agent in &report.agents {
            println!("{}: {}", agent.agent_id, agent.status);
        }
    }

    println!("\n=== decisions ===");
    if report.decisions.is_empty() {
        println!("(none)");
    } else {
        for decision in &report.decisions {
            println!("{}", serde_json::to_string(decision).unwrap_or_default());
        }
    }

    if !report.failures.is_empty() {
        println!("\nFAILURES: {}", report.failures.len());
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
