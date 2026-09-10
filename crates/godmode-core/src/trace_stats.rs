//! Built-in readers and aggregations for the observability JSONL trace.

use std::collections::{BTreeMap, HashSet};
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Aggregated duration statistics for one skill.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SkillDuration {
    /// Skill name from the trace event.
    pub skill: String,
    /// Number of completed runs.
    pub runs: usize,
    /// Integer average duration in milliseconds.
    pub avg_ms: u64,
    /// Maximum duration in milliseconds.
    pub max_ms: u64,
}

/// Latest lifecycle state observed for an agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum ConvergenceStatus {
    /// The agent has started and has no terminal event.
    Running,
    /// The agent completed its work.
    Complete,
    /// The agent stopped because it was blocked.
    Blocked,
}

impl ConvergenceStatus {
    /// Return the stable lowercase wire representation.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Complete => "complete",
            Self::Blocked => "blocked",
        }
    }
}

impl std::fmt::Display for ConvergenceStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl AsRef<str> for ConvergenceStatus {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl PartialEq<str> for ConvergenceStatus {
    fn eq(&self, other: &str) -> bool {
        matches!(
            (self, other),
            (Self::Running, "running") | (Self::Complete, "complete") | (Self::Blocked, "blocked")
        )
    }
}

impl PartialEq<&str> for ConvergenceStatus {
    fn eq(&self, other: &&str) -> bool {
        self == *other
    }
}

/// Latest observed convergence state for one agent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AgentConvergence {
    /// Agent identifier from the trace event.
    pub agent_id: String,
    /// One of `running`, `complete`, or `blocked`.
    pub status: ConvergenceStatus,
}

/// Combined statistics for the observability trace.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TraceStats {
    /// Skill duration aggregates sorted by skill name.
    pub skills: Vec<SkillDuration>,
    /// Agent convergence states in first-start order.
    pub agents: Vec<AgentConvergence>,
    /// Decision events in trace order.
    pub decisions: Vec<Value>,
    /// Failure events in trace order.
    pub failures: Vec<Value>,
    /// Valid legacy rows that do not contain an `event` field.
    pub legacy_rows: usize,
    /// Rows that could not be parsed as JSON objects.
    pub malformed_rows: usize,
}

/// Cross-session trace summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TraceSessionSummary {
    /// Session identifier.
    pub session_id: String,
    /// Timestamp of the first event in the session.
    pub started_at: String,
    /// Number of `skill.error` events.
    pub errors: usize,
    /// Number of agents whose latest lifecycle state is blocked.
    pub agents_blocked: usize,
    /// Number of governance denials in the session.
    pub agents_denied: usize,
    /// Number of agents whose latest lifecycle state is complete.
    pub agents_complete: usize,
    /// Number of started agents without a terminal event.
    pub agents_running: usize,
    /// Number of decision events.
    pub decisions: usize,
}

#[derive(Debug, Default)]
struct TraceLog {
    events: Vec<Value>,
    legacy_rows: usize,
    malformed_rows: usize,
}

/// Returns the last `limit` structured events, optionally scoped to a session.
///
/// # Errors
///
/// Returns an error when the trace file cannot be read. A zero limit returns no events.
///
/// # Examples
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use std::path::Path;
///
/// let events = godmode_core::trace_stats::tail(Path::new("."), 20, None)?;
/// println!("read {} events", events.len());
/// # Ok(())
/// # }
/// ```
pub fn tail(root: &Path, limit: usize, session: Option<&str>) -> Result<Vec<Value>> {
    if limit == 0 {
        return Ok(Vec::new());
    }
    tail_checked(root, limit, session)
}

/// Returns the last non-zero number of structured events, optionally scoped to a session.
pub fn tail_checked(root: &Path, limit: usize, session: Option<&str>) -> Result<Vec<Value>> {
    anyhow::ensure!(limit > 0, "trace tail limit must be greater than zero");
    let log = load(root)?;
    let events = filtered_events(&log, session);
    let start = events.len().saturating_sub(limit);
    Ok(events[start..]
        .iter()
        .map(|event| (*event).clone())
        .collect())
}

/// Returns all `skill.error`, `agent.blocked`, and `agent.denied` events.
///
/// # Examples
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use std::path::Path;
///
/// let failures = godmode_core::trace_stats::failures(Path::new("."), None)?;
/// println!("found {} failures", failures.len());
/// # Ok(())
/// # }
/// ```
pub fn failures(root: &Path, session: Option<&str>) -> Result<Vec<Value>> {
    let log = load(root)?;
    Ok(filtered_events(&log, session)
        .into_iter()
        .filter(|event| {
            matches!(
                event_name(event),
                Some("skill.error" | "agent.blocked" | "agent.denied")
            )
        })
        .cloned()
        .collect())
}

/// Aggregates skill durations, agent convergence, decisions, and failures.
///
/// # Examples
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use std::path::Path;
///
/// let report = godmode_core::trace_stats::stats(Path::new("."), None)?;
/// println!("aggregated {} skills", report.skills.len());
/// # Ok(())
/// # }
/// ```
pub fn stats(root: &Path, session: Option<&str>) -> Result<TraceStats> {
    let log = load(root)?;
    let events = filtered_events(&log, session);
    let mut durations: BTreeMap<String, Vec<u64>> = BTreeMap::new();
    let mut agent_order = Vec::new();
    let mut seen_agents = HashSet::new();
    let mut agent_states: BTreeMap<String, &str> = BTreeMap::new();
    let mut decisions = Vec::new();
    let mut failure_events = Vec::new();

    for event in events {
        match event_name(event) {
            Some("skill.complete") => {
                if let (Some(skill), Some(duration_ms)) = (
                    event.get("skill").and_then(Value::as_str),
                    event.get("duration_ms").and_then(Value::as_u64),
                ) {
                    durations
                        .entry(skill.to_owned())
                        .or_default()
                        .push(duration_ms);
                }
            }
            Some("agent.start") => {
                if let Some(agent_id) = event.get("agent_id").and_then(Value::as_str)
                    && event.get("slot").is_some()
                    && seen_agents.insert(agent_id.to_owned())
                {
                    agent_order.push(agent_id.to_owned());
                }
                if let Some(agent_id) = event.get("agent_id").and_then(Value::as_str)
                    && event.get("slot").is_some()
                {
                    agent_states.insert(agent_id.to_owned(), "running");
                }
            }
            Some("agent.complete") => {
                if let Some(agent_id) = event.get("agent_id").and_then(Value::as_str)
                    && event.get("slot").is_some()
                {
                    agent_states.insert(agent_id.to_owned(), "complete");
                }
            }
            Some("agent.blocked") => {
                if let Some(agent_id) = event.get("agent_id").and_then(Value::as_str)
                    && event.get("slot").is_some()
                {
                    agent_states.insert(agent_id.to_owned(), "blocked");
                }
                failure_events.push(event.clone());
            }
            Some("skill.error" | "agent.denied") => failure_events.push(event.clone()),
            Some("decision") => decisions.push(event.clone()),
            _ => {}
        }
    }

    let skills = durations
        .into_iter()
        .map(|(skill, values)| {
            let total: u64 = values.iter().sum();
            SkillDuration {
                skill,
                runs: values.len(),
                avg_ms: total / values.len() as u64,
                max_ms: values.iter().copied().max().unwrap_or_default(),
            }
        })
        .collect();
    let agents = agent_order
        .into_iter()
        .map(|agent_id| {
            let status = match agent_states.get(&agent_id).copied().unwrap_or("running") {
                "complete" => ConvergenceStatus::Complete,
                "blocked" => ConvergenceStatus::Blocked,
                _ => ConvergenceStatus::Running,
            };
            AgentConvergence { agent_id, status }
        })
        .collect();

    Ok(TraceStats {
        skills,
        agents,
        decisions,
        failures: failure_events,
        legacy_rows: log.legacy_rows,
        malformed_rows: log.malformed_rows,
    })
}

/// Reads the session identifier currently used by trace writers.
///
/// # Examples
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use std::path::Path;
///
/// let session_id = godmode_core::trace_stats::current_session_id(Path::new("."))?;
/// println!("current session: {session_id:?}");
/// # Ok(())
/// # }
/// ```
pub fn current_session_id(root: &Path) -> Result<Option<String>> {
    let path = root.join(".ctx/godmode/session.json");
    let raw = match std::fs::read_to_string(&path) {
        Ok(raw) => raw,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error).with_context(|| format!("reading {}", path.display())),
    };
    let value: Value =
        serde_json::from_str(&raw).with_context(|| format!("parsing {}", path.display()))?;
    Ok(value
        .get("session_id")
        .and_then(Value::as_str)
        .map(str::to_owned))
}

/// Summarizes the most recent session boundary groups.
///
/// # Examples
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use std::path::Path;
///
/// let summaries = godmode_core::trace_stats::summaries(Path::new("."), 3, None)?;
/// println!("summarized {} sessions", summaries.len());
/// # Ok(())
/// # }
/// ```
pub fn summaries(
    root: &Path,
    limit: usize,
    exclude_session: Option<&str>,
) -> Result<Vec<TraceSessionSummary>> {
    let log = load(root)?;
    let mut session_ids = Vec::new();
    let mut seen = HashSet::new();
    for event in &log.events {
        if matches!(event_name(event), Some("session.start" | "session.end"))
            && let Some(session_id) = event.get("session_id").and_then(Value::as_str)
            && Some(session_id) != exclude_session
            && seen.insert(session_id.to_owned())
        {
            session_ids.push(session_id.to_owned());
        }
    }
    let start = session_ids.len().saturating_sub(limit);
    Ok(session_ids[start..]
        .iter()
        .map(|session_id| summarize_session(&log.events, session_id))
        .collect())
}

fn load(root: &Path) -> Result<TraceLog> {
    let path = root.join(".ctx/godmode/traces/trace.jsonl");
    let raw = match std::fs::read_to_string(&path) {
        Ok(raw) => raw,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(TraceLog::default());
        }
        Err(error) => return Err(error).with_context(|| format!("reading {}", path.display())),
    };
    let mut log = TraceLog::default();
    for line in raw.lines().filter(|line| !line.trim().is_empty()) {
        match serde_json::from_str::<Value>(line) {
            Ok(value) if value.is_object() && value.get("event").is_none() => {
                log.legacy_rows += 1;
            }
            Ok(value) if value.get("event").and_then(Value::as_str).is_some() => {
                log.events.push(value);
            }
            _ => log.malformed_rows += 1,
        }
    }
    Ok(log)
}

fn filtered_events<'a>(log: &'a TraceLog, session: Option<&str>) -> Vec<&'a Value> {
    log.events
        .iter()
        .filter(|event| {
            session.is_none_or(|expected| {
                event.get("session_id").and_then(Value::as_str) == Some(expected)
            })
        })
        .collect()
}

fn event_name(event: &Value) -> Option<&str> {
    event.get("event").and_then(Value::as_str)
}

fn summarize_session(events: &[Value], session_id: &str) -> TraceSessionSummary {
    let scoped: Vec<&Value> = events
        .iter()
        .filter(|event| event.get("session_id").and_then(Value::as_str) == Some(session_id))
        .collect();
    let mut agent_states: BTreeMap<&str, &str> = BTreeMap::new();
    for event in &scoped {
        let Some(agent_id) = event.get("agent_id").and_then(Value::as_str) else {
            continue;
        };
        match event_name(event) {
            Some("agent.start") if event.get("slot").is_some() => {
                agent_states.insert(agent_id, "running");
            }
            Some("agent.complete") if event.get("slot").is_some() => {
                agent_states.insert(agent_id, "complete");
            }
            Some("agent.blocked") if event.get("slot").is_some() => {
                agent_states.insert(agent_id, "blocked");
            }
            _ => {}
        }
    }

    TraceSessionSummary {
        session_id: session_id.to_owned(),
        started_at: scoped
            .first()
            .and_then(|event| event.get("ts"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        errors: scoped
            .iter()
            .filter(|event| event_name(event) == Some("skill.error"))
            .count(),
        agents_blocked: agent_states
            .values()
            .filter(|status| **status == "blocked")
            .count(),
        agents_denied: scoped
            .iter()
            .filter(|event| event_name(event) == Some("agent.denied"))
            .count(),
        agents_complete: agent_states
            .values()
            .filter(|status| **status == "complete")
            .count(),
        agents_running: agent_states
            .values()
            .filter(|status| **status == "running")
            .count(),
        decisions: scoped
            .iter()
            .filter(|event| event_name(event) == Some("decision"))
            .count(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tail_checked_rejects_zero_limit() {
        let temp = tempfile::tempdir().unwrap();

        let error = tail_checked(temp.path(), 0, None).unwrap_err();

        assert_eq!(
            error.to_string(),
            "trace tail limit must be greater than zero"
        );
    }

    #[test]
    fn tail_limit_zero_returns_no_events() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join(".ctx/godmode/traces/trace.jsonl");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, serde_json::json!({"event": "decision"}).to_string()).unwrap();
        assert!(tail(root.path(), 0, None).unwrap().is_empty());
    }

    #[test]
    fn convergence_status_is_typed_and_string_compatible() {
        assert_eq!(ConvergenceStatus::Running, "running");
        assert_eq!(
            serde_json::to_string(&ConvergenceStatus::Complete).unwrap(),
            "\"complete\""
        );
    }

    #[test]
    fn trace_query_errors_include_the_trace_path() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join(".ctx/godmode/traces/trace.jsonl");
        std::fs::create_dir_all(&path).unwrap();
        for error in [
            tail(root.path(), 1, None).unwrap_err(),
            failures(root.path(), None).unwrap_err(),
            stats(root.path(), None).unwrap_err(),
            summaries(root.path(), 1, None).unwrap_err(),
        ] {
            assert!(error.to_string().contains("trace.jsonl"), "{error}");
        }
    }
}
