//! Policy enforcement: check tool calls and audit event I/O.

use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::policy::{CheckResult, GovernancePolicy, PolicyAction};

/// Check if a tool call is allowed by a resolved policy.
pub fn check_tool(
    policy: &GovernancePolicy,
    tool_name: &str,
    content: Option<&str>,
) -> CheckResult {
    // 1. Blocked tools
    if policy.blocked_tools.iter().any(|t| t == tool_name) {
        return CheckResult {
            action: PolicyAction::Deny,
            reason: format!("tool '{tool_name}' is in blocked_tools"),
        };
    }

    // 2. Allowed tools (if non-empty, tool must be in list)
    if !policy.allowed_tools.is_empty() && !policy.allowed_tools.iter().any(|t| t == tool_name) {
        return CheckResult {
            action: PolicyAction::Deny,
            reason: format!("tool '{tool_name}' not in allowed_tools"),
        };
    }

    // 3. Human approval
    if policy
        .require_human_approval
        .iter()
        .any(|a| a == "*" || a == tool_name)
    {
        return CheckResult {
            action: PolicyAction::Review,
            reason: format!("tool '{tool_name}' requires human approval"),
        };
    }

    // 4. Content patterns
    if let Some(text) = content {
        for pattern in &policy.blocked_patterns {
            match regex::Regex::new(pattern) {
                Ok(re) => {
                    if re.is_match(text) {
                        return CheckResult {
                            action: PolicyAction::Deny,
                            reason: format!("content matches blocked pattern: {pattern}"),
                        };
                    }
                }
                Err(_) => {
                    tracing::warn!("invalid regex in blocked_patterns: {pattern}");
                }
            }
        }
    }

    CheckResult {
        action: PolicyAction::Allow,
        reason: "passed all policy checks".to_string(),
    }
}

/// Serializable record of one governance policy decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceEvent {
    /// Event timestamp.
    pub ts: String,
    /// Event type identifier.
    pub event: String,
    /// Recorded policy action.
    pub action: String,
    /// Identifier of the agent that requested the tool call.
    pub agent_id: String,
    /// Name of the requested tool.
    pub tool_name: String,
    /// Human-readable reason for the decision.
    pub reason: String,
    #[serde(default)]
    /// Blocked pattern that matched, when applicable.
    pub pattern: String,
    #[serde(default)]
    /// Session identifier associated with the decision.
    pub session_id: String,
}

/// Append a governance audit event to the JSONL trail.
// qual:test_helper
pub fn emit_audit_event(root: &Path, event: &GovernanceEvent) -> Result<()> {
    let trace_dir = root.join(".ctx").join("godmode").join("traces");
    std::fs::create_dir_all(&trace_dir)?;
    let path = trace_dir.join("governance-audit.jsonl");
    let line = serde_json::to_string(event)?;
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    writeln!(file, "{line}")?;
    Ok(())
}

/// Read governance audit events, optionally filtered by date prefix.
pub fn read_audit_events(root: &Path, date_filter: Option<&str>) -> Result<Vec<GovernanceEvent>> {
    let path = root
        .join(".ctx")
        .join("godmode")
        .join("traces")
        .join("governance-audit.jsonl");
    if !path.exists() {
        return Ok(vec![]);
    }
    let raw = std::fs::read_to_string(&path)?;
    let mut events = Vec::new();
    for line in raw.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(ev) = serde_json::from_str::<GovernanceEvent>(line) {
            if date_filter.is_some_and(|d| !ev.ts.starts_with(d)) {
                continue;
            }
            events.push(ev);
        }
    }
    Ok(events)
}
