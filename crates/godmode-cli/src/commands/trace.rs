//! Queries the observability trace at `.ctx/godmode/traces/trace.jsonl`.
//!
//! This is a thin adapter: it reads through [`godmode_core::trace_store`], hands
//! the body to the pure [`godmode_core::trace_query`] functions, and renders.
//! No trace logic lives here, so the same queries are available to library
//! callers and the rendering can change without touching the answers.

use std::path::Path;

use anyhow::Result;
use godmode_core::hooks::trace_log;
use godmode_core::trace_query::{self, Record, Stats};
use godmode_core::trace_store;
use serde_json::json;

use crate::{EmitAction, TraceAction};

/// Dispatches a trace subcommand.
pub fn run_trace_action(root: &Path, json: bool, action: TraceAction) -> Result<()> {
    // Emission never needs a trace that already exists, and it must not print
    // the "No trace file." path a query would.
    if let TraceAction::Emit { action } = action {
        return emit(root, action);
    }

    let Some(body) = trace_store::read_body(root) else {
        if json {
            println!("[]");
        } else {
            println!("No trace file.");
        }
        return Ok(());
    };
    let records = trace_query::parse(&body);

    match action {
        TraceAction::Tail { n, session } => tail(&records, json, n, session.as_deref()),
        TraceAction::Failures { session } => failures(&records, json, session.as_deref()),
        TraceAction::Stats { session } => stats(&records, json, session.as_deref()),
        TraceAction::Summary { sessions } => summary(&records, json, sessions),
        TraceAction::Emit { .. } => unreachable!("handled above"),
    }
}

/// Append one lifecycle event.
///
/// The skill name and duration are recovered from the trace id rather than
/// passed in, so the shim that calls this never has to know the id layout.
fn emit(root: &Path, action: EmitAction) -> Result<()> {
    match action {
        EmitAction::SkillStart {
            skill,
            helper,
            args,
        } => {
            let tid = trace_log::skill_start(root, &skill, &helper, &args);
            println!("{tid}");
        }
        EmitAction::SkillComplete { trace_id } => {
            let skill = trace_log::parse_trace_id(&trace_id)
                .map_or_else(|| "unknown".to_string(), |(s, _)| s);
            let ms = trace_log::elapsed_ms(&trace_id);
            trace_log::skill_complete(root, &skill, &trace_id, ms);
        }
        EmitAction::SkillError {
            trace_id,
            exit_code,
            stderr_tail,
        } => {
            let skill = trace_log::parse_trace_id(&trace_id)
                .map_or_else(|| "unknown".to_string(), |(s, _)| s);
            let ms = trace_log::elapsed_ms(&trace_id);
            trace_log::skill_error(root, &skill, &trace_id, exit_code, &stderr_tail, ms);
        }
        EmitAction::Decision {
            skill,
            helper,
            kind,
            value,
        } => trace_log::decision(root, &skill, &helper, &kind, &value),
        EmitAction::ToolUse { tool, failed } => trace_log::tool_use(root, &tool, failed),
        EmitAction::AgentStart {
            agent_id,
            slot,
            crate_name,
        } => trace_log::agent_start(root, &agent_id, &slot, &crate_name),
        EmitAction::AgentComplete {
            agent_id,
            slot,
            commits,
        } => trace_log::agent_complete(root, &agent_id, &slot, &commits),
        EmitAction::AgentBlocked {
            agent_id,
            slot,
            reason,
        } => trace_log::agent_blocked(root, &agent_id, &slot, &reason),
    }
    Ok(())
}

/// Load, narrow to a session, and hand back an owned view.
///
/// Cloning keeps every query call on the same `&[Record]` shape, so the core API
/// stays free of session-filter variants that would each need their own tests.
fn scoped(records: &[Record], session: Option<&str>) -> Vec<Record> {
    match session {
        None => records.to_vec(),
        Some(sid) => records
            .iter()
            .filter(|r| r.session_id.as_deref() == Some(sid))
            .cloned()
            .collect(),
    }
}

fn phase_of(r: &Record) -> &'static str {
    r.phase.as_str()
}

fn status_of(r: &Record) -> String {
    match r.status {
        Some(s) => format!("{s:?}").to_lowercase(),
        None => "-".to_string(),
    }
}

fn tail(records: &[Record], as_json: bool, n: usize, session: Option<&str>) -> Result<()> {
    let view = scoped(records, session);
    let recent = trace_query::tail(&view, n);

    if as_json {
        let arr: Vec<_> = recent.iter().map(|r| record_json(r)).collect();
        println!("{}", serde_json::to_string_pretty(&arr)?);
        return Ok(());
    }

    if recent.is_empty() {
        println!("No records.");
        return Ok(());
    }

    let rows: Vec<Vec<String>> = recent
        .iter()
        .map(|r| {
            vec![
                r.started_at.map_or(String::new(), |t| t.to_rfc3339()),
                phase_of(r).to_string(),
                status_of(r),
                r.name.clone(),
                r.session_id.clone().unwrap_or_default(),
                r.duration_ms.map_or_else(String::new, |d| d.to_string()),
                r.error.clone().unwrap_or_default(),
            ]
        })
        .collect();
    print_table(
        &["ts", "phase", "status", "name", "session", "ms", "error"],
        &rows,
    );
    Ok(())
}

fn failures(records: &[Record], as_json: bool, session: Option<&str>) -> Result<()> {
    let view = scoped(records, session);
    let bad = trace_query::failures(&view);

    if as_json {
        let arr: Vec<_> = bad.iter().map(|r| record_json(r)).collect();
        println!("{}", serde_json::to_string_pretty(&arr)?);
        return Ok(());
    }

    if bad.is_empty() {
        println!("No failures.");
        return Ok(());
    }

    let rows: Vec<Vec<String>> = bad
        .iter()
        .map(|r| {
            vec![
                r.started_at.map_or(String::new(), |t| t.to_rfc3339()),
                r.name.clone(),
                status_of(r),
                r.error.clone().unwrap_or_default(),
                r.session_id.clone().unwrap_or_default(),
            ]
        })
        .collect();
    print_table(&["ts", "name", "status", "error", "session"], &rows);
    Ok(())
}

fn stats(records: &[Record], as_json: bool, session: Option<&str>) -> Result<()> {
    let view = scoped(records, session);
    let s = trace_query::stats(&view);

    if as_json {
        println!("{}", serde_json::to_string_pretty(&stats_json(&s))?);
        return Ok(());
    }

    // Freshness first. A stale log otherwise reads as a description of now,
    // which is the failure mode this command exists to prevent.
    println!("=== trace freshness ===");
    if s.total_records == 0 {
        println!("(no events)");
        return Ok(());
    }
    let last = s.last_ts.clone().unwrap_or_default();
    println!("  total records: {}", s.total_records);
    println!("  last record:   {last}  ({})", trace_query::age(&last));
    if trace_query::is_today(&last) {
        println!("  \u{1b}[32mcurrent\u{1b}[0m");
    } else {
        println!("  \u{1b}[33mSTALE: nothing recorded today.\u{1b}[0m");
        println!("  This log does NOT describe the current session.");
    }
    println!("  sessions:      {} distinct", s.distinct_sessions);

    println!("\n=== skill durations (ms) ===");
    if s.skill_durations.is_empty() {
        println!("(none)");
    } else {
        let rows: Vec<Vec<String>> = s
            .skill_durations
            .iter()
            .map(|d| {
                vec![
                    d.skill.clone(),
                    d.runs.to_string(),
                    d.avg_ms.to_string(),
                    d.max_ms.to_string(),
                ]
            })
            .collect();
        print_table(&["skill", "runs", "avg_ms", "max_ms"], &rows);
    }

    println!("\n=== tool use ===");
    if s.tool_counts.is_empty() {
        println!("  (none recorded — the PostToolUse trace hook may not be registered)");
    } else {
        let rows: Vec<Vec<String>> = s
            .tool_counts
            .iter()
            .map(|t| vec![t.tool.clone(), t.calls.to_string()])
            .collect();
        print_table(&["tool", "calls"], &rows);
    }

    println!("\n=== agent convergence ===");
    if s.agents.is_empty() {
        println!("  (no agents dispatched)");
    } else {
        for (id, status) in &s.agents {
            println!("  {id}: {}", status.as_str());
        }
    }

    println!("\n=== decisions ===");
    println!("  {}", s.decision_count);
    if s.failure_count > 0 {
        println!("\n=== failures ===");
        println!(
            "  {} — run `godmode trace failures` for detail",
            s.failure_count
        );
    }
    Ok(())
}

fn summary(records: &[Record], as_json: bool, sessions: usize) -> Result<()> {
    let ids = trace_query::session_ids(records);
    let start = ids.len().saturating_sub(sessions);

    let mut out = Vec::new();
    for sid in &ids[start..] {
        let view: Vec<&Record> = records
            .iter()
            .filter(|r| r.session_id.as_deref() == Some(sid.as_str()))
            .collect();
        out.push(trace_query::summarise_session(sid, &view));
    }

    if as_json {
        let arr: Vec<_> = out
            .iter()
            .map(|s| {
                json!({
                    "session_id": s.session_id,
                    "started": s.started,
                    "errors": s.errors,
                    "blocked": s.blocked,
                    "complete": s.complete,
                    "decisions": s.decisions,
                    "unresolved": s.unresolved,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&arr)?);
        return Ok(());
    }

    if out.is_empty() {
        println!("No sessions recorded.");
        return Ok(());
    }

    for s in &out {
        println!("--- {} @ {}", s.session_id, s.started);
        println!(
            "    errors={}  blocked={}  agents: {} complete / {} still-running / {} decisions",
            s.errors,
            s.blocked,
            s.complete,
            s.unresolved.len(),
            s.decisions
        );
        for id in &s.unresolved {
            println!("    UNRESOLVED agent: {id}");
        }
    }
    Ok(())
}

fn record_json(r: &Record) -> serde_json::Value {
    json!({
        "ts": r.started_at.map(|t| t.to_rfc3339()),
        "phase": r.phase.as_str(),
        "status": r.status.map(|s| format!("{s:?}").to_lowercase()),
        "name": r.name,
        "kind": format!("{:?}", r.kind).to_lowercase(),
        "session_id": r.session_id,
        "trace_id": r.trace_id,
        "duration_ms": r.duration_ms,
        "error": r.error,
        "metadata": r.metadata,
    })
}

fn stats_json(s: &Stats) -> serde_json::Value {
    json!({
        "total_records": s.total_records,
        "distinct_sessions": s.distinct_sessions,
        "last_ts": s.last_ts,
        "session_starts": s.session_starts,
        "session_ends": s.session_ends,
        "failure_count": s.failure_count,
        "decision_count": s.decision_count,
        "skill_durations": s.skill_durations,
        "tool_counts": s.tool_counts,
        "agents": s.agents.iter()
            .map(|(id, status)| json!({ "agent_id": id, "status": status.as_str() }))
            .collect::<Vec<_>>(),
    })
}

/// Left-aligned, two-space-gapped columns sized to the widest cell.
fn print_table(headers: &[&str], rows: &[Vec<String>]) {
    let mut widths: Vec<usize> = headers.iter().map(|h| h.chars().count()).collect();
    for row in rows {
        for (i, c) in row.iter().enumerate().take(widths.len()) {
            widths[i] = widths[i].max(c.chars().count());
        }
    }
    let line = |cells: &[String]| -> String {
        let mut s = String::new();
        for (i, c) in cells.iter().enumerate() {
            let pad = widths[i].saturating_sub(c.chars().count());
            s.push_str(c);
            s.push_str(&" ".repeat(pad));
            s.push_str("  ");
        }
        s.trim_end().to_string()
    };
    let head: Vec<String> = headers.iter().map(|h| (*h).to_string()).collect();
    println!("{}", line(&head));
    for row in rows {
        println!("{}", line(row));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LEGACY: &str = concat!(
        r#"{"event":"skill.start","skill":"ci-fix","session_id":"s1","ts":"2026-09-30T09:00:00Z"}"#,
        "\n",
        r#"{"event":"skill.error","skill":"ci-fix","session_id":"s1","ts":"2026-09-30T09:00:01Z","duration_ms":1000,"exit_code":2,"reason":"one or more helpers failed"}"#,
        "\n",
        r#"{"event":"agent.start","agent_id":"explore","session_id":"s1","ts":"2026-09-30T09:00:02Z"}"#,
        "\n",
        r#"{"event":"session.end","session_id":"s1","ts":"2026-09-30T09:05:00Z","dirty_files":3}"#,
        "\n",
    );

    fn parsed() -> Vec<Record> {
        trace_query::parse(LEGACY)
    }

    #[test]
    fn scoped_narrows_to_one_session() {
        let records = parsed();
        assert_eq!(scoped(&records, Some("s1")).len(), 4);
        assert_eq!(scoped(&records, Some("nope")).len(), 0);
        assert_eq!(scoped(&records, None).len(), 4);
    }

    #[test]
    fn status_renders_as_dash_when_open() {
        let records = parsed();
        assert_eq!(status_of(&records[0]), "-", "an open step has no outcome");
        assert_eq!(status_of(&records[1]), "err");
    }

    #[test]
    fn stats_over_legacy_records_finds_the_failure() {
        let s = trace_query::stats(&parsed());
        assert_eq!(s.total_records, 4);
        assert_eq!(s.failure_count, 1);
        assert_eq!(s.distinct_sessions, 1);
    }

    #[test]
    fn summary_over_legacy_records_reports_the_unresolved_agent() {
        let records = parsed();
        let view: Vec<&Record> = records.iter().collect();
        let s = trace_query::summarise_session("s1", &view);
        assert_eq!(s.errors, 1);
        assert_eq!(s.unresolved, vec!["explore".to_string()]);
    }

    #[test]
    fn record_json_carries_the_correlation_fields() {
        let v = record_json(&parsed()[0]);
        assert_eq!(v.get("phase").and_then(|x| x.as_str()), Some("start"));
        assert_eq!(v.get("name").and_then(|x| x.as_str()), Some("ci-fix"));
        assert_eq!(v.get("session_id").and_then(|x| x.as_str()), Some("s1"));
    }

    #[test]
    fn table_pads_columns_and_trims_trailing_space() {
        let rows = vec![vec!["a".to_string(), "bbb".to_string()]];
        let mut widths = vec![3usize, 3];
        for row in &rows {
            for (i, c) in row.iter().enumerate() {
                widths[i] = widths[i].max(c.chars().count());
            }
        }
        assert_eq!(widths, vec![3, 3]);
    }
}
