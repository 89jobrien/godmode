//! Read side of the godmode trace log.
//!
//! The log is `.ctx/godmode/traces/trace.jsonl`: an append-only, best-effort,
//! cross-process event stream written by Nushell hooks, Rust hooks, and the CLI
//! alike. This module turns those lines back into typed records and answers the
//! questions the `godmode trace` subcommands ask.
//!
//! # Why crux `Step`
//!
//! New lines are written in crux's canonical [`Step`] schema so the event
//! vocabulary is owned by crux rather than re-declared in godmode and Nushell
//! separately. `Step` is already serialisable, so the on-disk form and the
//! in-memory form are the same type.
//!
//! Two fields carry meaning that crux's own enums do not model, and both live
//! in `Step::metadata` — the extension point crux provides:
//!
//! * `phase` — `start` or `terminal`. A `Step` describes a *resolved* step, but
//!   a JSONL log records both halves of a lifecycle as separate lines. The
//!   unresolved-agent query depends on seeing the open half.
//! * `session_id` / `trace_id` / `slot` / `crate` — correlation fields.
//!
//! What this module deliberately does **not** do is wrap each line in a
//! `Crux<T>` envelope. `Crux<T>` models one replayable computation owned by one
//! process; this log is written by many short-lived processes that share
//! nothing but the file, and emitting is explicitly non-fatal. Enforcing a
//! resolved-value shape on a fire-and-forget audit line would be ceremony that
//! breaks the guarantee the hot path relies on. `session_trace` remains the
//! place `Crux<T>` belongs.

use std::collections::{BTreeMap, HashSet};

use chrono::{DateTime, FixedOffset, Local};
use crux_runtime::types::step::{Step, StepKind, StepStatus};
use serde_json::Value;

/// Which half of a lifecycle a line represents.
///
/// Owned by [`trace_log`](crate::hooks::trace_log) because the writer defines
/// the vocabulary; this module only has to agree with it.
pub use crate::hooks::trace_log::Phase;

/// One trace line, normalised across both on-disk schemas.
#[derive(Debug, Clone)]
pub struct Record {
    /// Actor identity: skill name, agent id, or tool name.
    pub name: String,
    pub kind: StepKind,
    /// `None` while the lifecycle half is still open.
    pub status: Option<StepStatus>,
    pub phase: Phase,
    pub started_at: Option<DateTime<FixedOffset>>,
    pub duration_ms: Option<u64>,
    pub session_id: Option<String>,
    pub trace_id: Option<String>,
    /// Everything else the writer attached, including `slot`, `crate`,
    /// `exit_code`, and `stderr_tail`.
    pub metadata: BTreeMap<String, Value>,
    pub error: Option<String>,
}

impl Record {
    /// True when the record represents a failure: an errored step, or one that
    /// was rejected/blocked.
    pub fn is_failure(&self) -> bool {
        matches!(
            self.status,
            Some(StepStatus::Err) | Some(StepStatus::Rejected)
        )
    }

    /// True when the record opened a lifecycle that never closed.
    pub fn is_open(&self) -> bool {
        self.phase == Phase::Start
    }

    pub fn meta_str(&self, key: &str) -> Option<&str> {
        self.metadata.get(key).and_then(Value::as_str)
    }

    /// True when the record describes a dispatched agent rather than a skill
    /// or tool. crux spells this `Delegation`.
    pub fn is_agent(&self) -> bool {
        self.kind == StepKind::Delegation
    }

    /// True when the record is a branching decision. crux spells this `Branch`.
    pub fn is_decision(&self) -> bool {
        self.kind == StepKind::Branch
    }
}

/// Per-skill duration rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct SkillDuration {
    pub skill: String,
    pub runs: usize,
    pub avg_ms: u64,
    pub max_ms: u64,
}

/// Per-tool call count.
#[derive(Debug, Clone, PartialEq)]
pub struct ToolCount {
    pub tool: String,
    pub calls: usize,
}

/// Convergence status for one agent identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentStatus {
    Complete,
    Blocked,
    Running,
}

impl AgentStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            AgentStatus::Complete => "complete",
            AgentStatus::Blocked => "blocked",
            AgentStatus::Running => "running",
        }
    }
}

/// Aggregation behind `godmode trace stats`.
#[derive(Debug, Clone, Default)]
pub struct Stats {
    pub total_records: usize,
    pub distinct_sessions: usize,
    pub session_starts: usize,
    pub session_ends: usize,
    pub last_ts: Option<String>,
    pub last_at: Option<DateTime<FixedOffset>>,
    pub skill_durations: Vec<SkillDuration>,
    pub tool_counts: Vec<ToolCount>,
    pub agents: Vec<(String, AgentStatus)>,
    pub decision_count: usize,
    pub failure_count: usize,
}

/// Cross-session rollup behind `godmode trace summary`.
#[derive(Debug, Clone, PartialEq)]
pub struct SessionSummary {
    pub session_id: String,
    pub started: String,
    pub errors: usize,
    pub blocked: usize,
    pub complete: usize,
    pub decisions: usize,
    /// Agent identities whose `agent.start` has no terminal counterpart.
    pub unresolved: Vec<String>,
}

/// Parse a log body into records, skipping blank and unparseable lines.
///
/// A torn trailing line is expected whenever a writer is killed mid-append, so
/// one bad line must not cost the caller the rest of the file.
///
/// This module performs no I/O. Use [`trace_store::read_body`](crate::trace_store::read_body)
/// to obtain the text, then hand it here.
pub fn parse(body: &str) -> Vec<Record> {
    body.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .map(record_from_value)
        .collect()
}

/// Project one JSON line onto [`Record`], accepting both schemas.
///
/// The `Crux` branch is a `Step` that already carries its phase in metadata.
/// The legacy branch maps the pre-0.7 `{event, ...}` vocabulary, so logs written
/// before the upgrade keep answering queries.
fn record_from_value(value: Value) -> Record {
    match serde_json::from_value::<Step>(value.clone()) {
        Ok(step) => from_step(step),
        Err(_) => from_legacy(value),
    }
}

fn from_step(step: Step) -> Record {
    // crux types `metadata` as a HashMap; `Record` uses a BTreeMap so rendering
    // and tests see a deterministic key order.
    let metadata: BTreeMap<String, Value> = step.metadata.into_iter().collect();
    let session_id = meta_string(&metadata, "session_id");
    let trace_id = meta_string(&metadata, "trace_id");
    let phase = match meta_string(&metadata, "phase").as_deref() {
        Some("start") => Phase::Start,
        _ => Phase::Terminal,
    };
    // `phase` is authoritative over `status` for an open step. crux has no
    // "pending" variant, so the writer fills `Ok` on a start line as a schema
    // formality; treating that as an outcome would make a still-running skill
    // indistinguishable from a finished one.
    let status = if phase == Phase::Start {
        None
    } else {
        Some(step.status)
    };

    Record {
        name: step.name,
        kind: step.kind,
        status,
        phase,
        started_at: Some(step.started_at.into()),
        duration_ms: Some(step.duration_ms),
        session_id,
        trace_id,
        metadata,
        error: step.error,
    }
}

fn from_legacy(value: Value) -> Record {
    let event = string_at(&value, "event").unwrap_or_default();
    let status = status_for_event(&event, &value);
    let kind = kind_for_event(&event);
    let name = string_at(&value, "skill")
        .or_else(|| string_at(&value, "agent_id"))
        .or_else(|| string_at(&value, "tool"))
        .or_else(|| string_at(&value, "session_id"))
        .unwrap_or_default();
    let phase = if event.ends_with(".start") {
        Phase::Start
    } else {
        Phase::Terminal
    };

    let known = [
        "event",
        "skill",
        "agent_id",
        "tool",
        "session_id",
        "trace_id",
        "ts",
        "duration_ms",
        "reason",
        "failed",
        "kind",
        "value",
    ];
    // Everything the writer attached beyond the fields projected above survives
    // in metadata, so queries never have to know which schema produced a record.
    let mut metadata: BTreeMap<String, Value> = value
        .as_object()
        .map(|m| {
            m.iter()
                .filter(|(k, _)| !known.contains(&k.as_str()))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        })
        .unwrap_or_default();
    if metadata.is_empty() && value.as_object().is_some() {
        metadata.insert("event".into(), Value::String(event.clone()));
    }

    Record {
        name,
        kind,
        status,
        phase,
        started_at: string_at(&value, "ts").and_then(|s| DateTime::parse_from_rfc3339(&s).ok()),
        duration_ms: value.get("duration_ms").and_then(Value::as_u64),
        session_id: string_at(&value, "session_id"),
        trace_id: string_at(&value, "trace_id"),
        metadata,
        error: string_at(&value, "reason"),
    }
}

fn kind_for_event(event: &str) -> StepKind {
    match event {
        "agent.start" | "agent.complete" | "agent.blocked" => StepKind::Delegation,
        "decision" => StepKind::Branch,
        _ => StepKind::Plain,
    }
}

fn status_for_event(event: &str, value: &Value) -> Option<StepStatus> {
    match event {
        "skill.start" | "agent.start" => None,
        "skill.error" => Some(StepStatus::Err),
        "agent.blocked" => Some(StepStatus::Rejected),
        "tool.use" => Some(if value.get("failed") == Some(&Value::Bool(true)) {
            StepStatus::Err
        } else {
            StepStatus::Ok
        }),
        _ => Some(StepStatus::Ok),
    }
}

fn string_at(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(str::to_string)
}

fn meta_string(metadata: &BTreeMap<String, Value>, key: &str) -> Option<String> {
    metadata
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
}

/// The most recent `n` records, in file order.
pub fn tail(records: &[Record], n: usize) -> Vec<&Record> {
    let start = records.len().saturating_sub(n);
    records[start..].iter().collect()
}

/// Every failure, in file order.
pub fn failures(records: &[Record]) -> Vec<&Record> {
    records.iter().filter(|r| r.is_failure()).collect()
}

/// Distinct session ids, in first-seen order.
pub fn session_ids(records: &[Record]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for r in records {
        if let Some(sid) = &r.session_id
            && !out.contains(sid)
        {
            out.push(sid.clone());
        }
    }
    out
}

/// Agent identities dispatched with no terminal counterpart.
///
/// The terminal that resolves an agent is `agent.complete` or `agent.blocked`,
/// both of which carry the same `agent_id` as the opening `agent.start`.
pub fn unresolved_agents(records: &[&Record]) -> Vec<String> {
    let closed: HashSet<&str> = records
        .iter()
        .filter(|r| r.is_agent() && !r.is_open())
        .map(|r| r.name.as_str())
        .collect();

    // Order-independent: a terminal line that lands after its opening line must
    // still resolve it, so membership is tested against the whole set rather
    // than consumed positionally.
    let mut out: Vec<String> = Vec::new();
    for r in records.iter().filter(|r| r.is_agent() && r.is_open()) {
        if !closed.contains(r.name.as_str()) && !out.contains(&r.name) {
            out.push(r.name.clone());
        }
    }
    out
}

/// Roll up a single session.
pub fn summarise_session(session_id: &str, records: &[&Record]) -> SessionSummary {
    let starts: Vec<&&Record> = records.iter().filter(|r| r.is_open()).collect();
    SessionSummary {
        session_id: session_id.to_string(),
        started: starts
            .first()
            .and_then(|r| r.started_at)
            .map(|t| t.to_rfc3339())
            .unwrap_or_default(),
        errors: records
            .iter()
            .filter(|r| r.status == Some(StepStatus::Err))
            .count(),
        blocked: records
            .iter()
            .filter(|r| r.status == Some(StepStatus::Rejected))
            .count(),
        complete: records
            .iter()
            .filter(|r| r.is_agent() && !r.is_open() && r.status == Some(StepStatus::Ok))
            .count(),
        decisions: records.iter().filter(|r| r.is_decision()).count(),
        unresolved: unresolved_agents(records),
    }
}

/// Aggregate a record set.
pub fn stats(records: &[Record]) -> Stats {
    let mut durations: BTreeMap<String, Vec<u64>> = BTreeMap::new();
    let mut tools: BTreeMap<String, usize> = BTreeMap::new();

    for r in records {
        match (r.is_open(), r.duration_ms) {
            (false, Some(ms)) if !r.name.is_empty() && !r.is_agent() && !r.is_decision() => {
                durations.entry(r.name.clone()).or_default().push(ms);
            }
            _ => {}
        }
        if !r.is_open() && r.phase == Phase::Terminal && r.meta_str("tool").is_some() {
            *tools
                .entry(
                    r.meta_str("tool")
                        .map(str::to_string)
                        .unwrap_or_else(|| r.name.clone()),
                )
                .or_default() += 1;
        }
    }

    let mut agents: Vec<(String, AgentStatus)> = Vec::new();
    for id in records
        .iter()
        .filter(|r| r.is_agent() && r.is_open())
        .map(|r| r.name.clone())
    {
        if agents.iter().any(|(n, _)| *n == id) {
            continue;
        }
        let scoped: Vec<&Record> = records.iter().filter(|r| r.is_agent()).collect();
        let status = if scoped
            .iter()
            .any(|r| r.status == Some(StepStatus::Rejected) && r.name == id)
        {
            AgentStatus::Blocked
        } else if scoped
            .iter()
            .any(|r| r.status == Some(StepStatus::Ok) && r.name == id)
        {
            AgentStatus::Complete
        } else {
            AgentStatus::Running
        };
        agents.push((id, status));
    }

    let last = records.last();
    let mut skill_durations: Vec<SkillDuration> = durations
        .into_iter()
        .map(|(skill, runs)| {
            let sum: u64 = runs.iter().sum();
            let max = runs.iter().copied().max().unwrap_or_default();
            SkillDuration {
                skill,
                runs: runs.len(),
                avg_ms: sum / runs.len() as u64,
                max_ms: max,
            }
        })
        .collect();
    skill_durations.sort_by(|a, b| a.skill.cmp(&b.skill));

    let mut tool_counts: Vec<ToolCount> = tools
        .into_iter()
        .map(|(tool, calls)| ToolCount { tool, calls })
        .collect();
    tool_counts.sort_by(|a, b| b.calls.cmp(&a.calls).then(a.tool.cmp(&b.tool)));

    Stats {
        total_records: records.len(),
        distinct_sessions: session_ids(records).len(),
        session_starts: records
            .iter()
            .filter(|r| r.is_open() && r.name.is_empty())
            .count(),
        session_ends: records
            .iter()
            .filter(|r| !r.is_open() && r.status == Some(StepStatus::Ok) && r.name.is_empty())
            .count(),
        last_ts: last.map(|r| r.started_at.map(|t| t.to_rfc3339()).unwrap_or_default()),
        last_at: last.and_then(|r| r.started_at),
        skill_durations,
        tool_counts,
        agents,
        decision_count: records.iter().filter(|r| r.is_decision()).count(),
        failure_count: failures(records).len(),
    }
}

/// Human-readable age of a timestamp, e.g. `7d old`.
pub fn age(ts: &str) -> String {
    match DateTime::parse_from_rfc3339(ts) {
        Err(_) => "unparseable".to_string(),
        Ok(parsed) => {
            let days = (Local::now() - parsed.with_timezone(&Local)).num_days();
            if days < 1 {
                "< 1d old".to_string()
            } else {
                format!("{days}d old")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rec(json: &str) -> Record {
        record_from_value(serde_json::from_str(json).expect("valid json"))
    }

    // --- crux schema -----------------------------------------------------

    fn crux_step_json(phase: &str) -> String {
        format!(
            r#"{{"name":"ci-fix","kind":"plain","status":"ok","confidence":1.0,
                 "started_at":"2026-09-30T09:00:00+00:00","duration_ms":120,
                 "input_hash":0,"attempt":1,"metadata":{{"phase":"{phase}",
                 "session_id":"s1","trace_id":"t1"}}}}"#
        )
    }

    #[test]
    fn crux_step_carries_phase_from_metadata() {
        assert_eq!(rec(&crux_step_json("start")).phase, Phase::Start);
        assert_eq!(rec(&crux_step_json("terminal")).phase, Phase::Terminal);
    }

    #[test]
    fn crux_step_surfaces_session_and_trace_ids() {
        let r = rec(&crux_step_json("terminal"));
        assert_eq!(r.session_id.as_deref(), Some("s1"));
        assert_eq!(r.trace_id.as_deref(), Some("t1"));
        assert_eq!(r.kind, StepKind::Plain);
        assert_eq!(r.status, Some(StepStatus::Ok));
    }

    // --- legacy schema ---------------------------------------------------

    #[test]
    fn legacy_agent_start_is_open_and_carries_no_status() {
        let r = rec(r#"{"event":"agent.start","agent_id":"explore","session_id":"s1"}"#);
        assert!(r.is_agent());
        assert!(r.is_open());
        assert_eq!(r.status, None, "an open step has no outcome yet");
        assert_eq!(r.name, "explore");
    }

    #[test]
    fn legacy_agent_blocked_is_rejected_not_err() {
        let r = rec(r#"{"event":"agent.blocked","agent_id":"a","reason":"nope"}"#);
        assert_eq!(r.status, Some(StepStatus::Rejected));
        assert!(r.is_failure());
        assert_eq!(r.error.as_deref(), Some("nope"));
    }

    #[test]
    fn legacy_skill_error_is_err() {
        let r = rec(r#"{"event":"skill.error","skill":"cron-refresh","duration_ms":9}"#);
        assert_eq!(r.status, Some(StepStatus::Err));
        assert!(r.is_failure());
        assert_eq!(r.duration_ms, Some(9));
    }

    #[test]
    fn legacy_tool_use_failure_maps_to_err() {
        let ok = rec(r#"{"event":"tool.use","tool":"Bash","failed":false}"#);
        let bad = rec(r#"{"event":"tool.use","tool":"Bash","failed":true}"#);
        assert_eq!(ok.status, Some(StepStatus::Ok));
        assert_eq!(bad.status, Some(StepStatus::Err));
        assert!(bad.is_failure());
    }

    #[test]
    fn legacy_decision_is_a_branch() {
        let r = rec(r#"{"event":"decision","skill":"ci-fix","kind":"ci_class","value":"flaky"}"#);
        assert!(r.is_decision());
        assert_eq!(r.kind, StepKind::Branch);
    }

    #[test]
    fn legacy_slot_survives_into_metadata() {
        // `slot` is documented on agent.blocked but never emitted on
        // agent.start. Reading it must never be an error, only an absent field.
        let r = rec(r#"{"event":"agent.blocked","agent_id":"a","slot":"2"}"#);
        assert_eq!(r.meta_str("slot"), Some("2"));
        let start = rec(r#"{"event":"agent.start","agent_id":"a"}"#);
        assert_eq!(start.meta_str("slot"), None);
    }

    #[test]
    fn legacy_record_without_session_id_parses() {
        let r = rec(r#"{"event":"hook_observed","ts":"2026-01-01T00:00:00Z"}"#);
        assert!(r.session_id.is_none());
        assert!(r.started_at.is_some());
    }

    // --- parsing ---------------------------------------------------------

    #[test]
    fn parse_skips_blank_torn_and_malformed_lines() {
        let body = concat!(
            r#"{"event":"skill.start","skill":"a","session_id":"s1"}"#,
            "\n",
            "\n",
            r#"{"event":"skill.comp"#,
            "\n",
            r#"{"event":"skill.complete","skill":"a","session_id":"s1"}"#,
            "\n",
        );
        let records = parse(body);
        assert_eq!(records.len(), 2, "a torn line must not cost the rest");
    }

    #[test]
    fn parse_of_empty_body_is_empty() {
        assert!(parse("").is_empty());
    }

    // --- queries ---------------------------------------------------------

    fn agent_fixture() -> Vec<Record> {
        vec![
            rec(r#"{"event":"agent.start","agent_id":"a","session_id":"s1"}"#),
            rec(r#"{"event":"agent.complete","agent_id":"a","session_id":"s1"}"#),
            rec(r#"{"event":"agent.start","agent_id":"b","session_id":"s1"}"#),
            rec(r#"{"event":"agent.blocked","agent_id":"b","session_id":"s1"}"#),
            rec(r#"{"event":"agent.start","agent_id":"c","session_id":"s1"}"#),
        ]
    }

    #[test]
    fn unresolved_agents_excludes_closed_identities() {
        let records = agent_fixture();
        let refs: Vec<&Record> = records.iter().collect();
        assert_eq!(unresolved_agents(&refs), vec!["c".to_string()]);
    }

    #[test]
    fn repeated_dispatch_of_one_id_yields_one_unresolved_entry() {
        // The live trace has six `subagent-explore` starts, all unresolved.
        // Convergence is per identity, not per record.
        let records = [
            rec(r#"{"event":"agent.start","agent_id":"x"}"#),
            rec(r#"{"event":"agent.start","agent_id":"x"}"#),
            rec(r#"{"event":"agent.start","agent_id":"x"}"#),
        ];
        let refs: Vec<&Record> = records.iter().collect();
        assert_eq!(unresolved_agents(&refs), vec!["x".to_string()]);
    }

    #[test]
    fn late_completion_resolves_an_earlier_open_step() {
        // Order in a JSONL log is not guaranteed to be start-then-terminal.
        let records = [
            rec(r#"{"event":"agent.complete","agent_id":"a"}"#),
            rec(r#"{"event":"agent.start","agent_id":"a"}"#),
        ];
        let refs: Vec<&Record> = records.iter().collect();
        assert!(unresolved_agents(&refs).is_empty());
    }

    #[test]
    fn session_summary_counts_each_outcome_class() {
        let records = [
            rec(r#"{"event":"skill.start","skill":"a","session_id":"s1"}"#),
            rec(r#"{"event":"skill.error","skill":"a","session_id":"s1"}"#),
            rec(r#"{"event":"agent.blocked","agent_id":"b","session_id":"s1"}"#),
            rec(r#"{"event":"decision","skill":"a","session_id":"s1"}"#),
        ];
        let refs: Vec<&Record> = records.iter().collect();
        let s = summarise_session("s1", &refs);
        assert_eq!(s.errors, 1);
        assert_eq!(s.blocked, 1);
        assert_eq!(s.decisions, 1);
        assert!(s.unresolved.is_empty());
    }

    #[test]
    fn session_ids_preserve_first_seen_order() {
        let records = vec![
            rec(r#"{"event":"skill.start","skill":"a","session_id":"s2"}"#),
            rec(r#"{"event":"skill.start","skill":"a","session_id":"s1"}"#),
            rec(r#"{"event":"skill.start","skill":"a","session_id":"s2"}"#),
        ];
        assert_eq!(session_ids(&records), vec!["s2", "s1"]);
    }

    #[test]
    fn stats_rolls_up_skill_durations() {
        let records = vec![
            rec(r#"{"event":"skill.complete","skill":"a","duration_ms":100}"#),
            rec(r#"{"event":"skill.complete","skill":"a","duration_ms":300}"#),
        ];
        let s = stats(&records);
        assert_eq!(s.skill_durations.len(), 1);
        let d = &s.skill_durations[0];
        assert_eq!(d.runs, 2);
        assert_eq!(d.avg_ms, 200);
        assert_eq!(d.max_ms, 300);
    }

    #[test]
    fn stats_ignores_durations_from_open_records() {
        // A start line can carry a duration field without having finished.
        let records = vec![rec(
            r#"{"event":"skill.start","skill":"a","duration_ms":999}"#,
        )];
        assert!(stats(&records).skill_durations.is_empty());
    }

    #[test]
    fn stats_classifies_agent_convergence() {
        let s = stats(&agent_fixture());
        let by_name = |n: &str| s.agents.iter().find(|(x, _)| x == n).map(|(_, v)| *v);
        assert_eq!(by_name("a"), Some(AgentStatus::Complete));
        assert_eq!(by_name("b"), Some(AgentStatus::Blocked));
        assert_eq!(by_name("c"), Some(AgentStatus::Running));
    }

    #[test]
    fn stats_counts_failures_across_both_statuses() {
        let s = stats(&agent_fixture());
        assert_eq!(s.failure_count, 1, "only the blocked agent is a failure");
    }

    #[test]
    fn tail_returns_most_recent_in_file_order() {
        let records = vec![
            rec(r#"{"event":"skill.start","skill":"a"}"#),
            rec(r#"{"event":"skill.start","skill":"b"}"#),
            rec(r#"{"event":"skill.start","skill":"c"}"#),
        ];
        let t = tail(&records, 2);
        assert_eq!(t.len(), 2);
        assert_eq!(t[0].name, "b");
        assert_eq!(t[1].name, "c");
    }

    #[test]
    fn tail_larger_than_input_returns_everything() {
        let records = vec![rec(r#"{"event":"skill.start","skill":"a"}"#)];
        assert_eq!(tail(&records, 99).len(), 1);
    }

    #[test]
    fn age_of_unparseable_timestamp_is_labelled() {
        assert_eq!(age("not a timestamp"), "unparseable");
    }
}
