//! agent-governance — PreToolUse/Agent hook.
//! Resolves governance policy for the dispatched agent and enforces constraints.
//! Outputs JSON: {"decision": "approve"|"block", "reason": "..."}

use std::path::Path;

use serde_json::{Value, json};

use super::trace_log;
use crate::agent;
use crate::policy;

/// Decision returned by the governance check.
#[derive(Debug)]
pub struct GovernanceDecision {
    /// Whether policy permits the requested agent dispatch.
    pub approved: bool,
    /// Human-readable explanation for the decision.
    pub reason: String,
    /// Policy constraints that the approved agent should observe.
    pub reminders: Vec<String>,
}

impl GovernanceDecision {
    fn approve(reason: impl Into<String>, reminders: Vec<String>) -> Self {
        Self {
            approved: true,
            reason: reason.into(),
            reminders,
        }
    }

    fn block(reason: impl Into<String>) -> Self {
        Self {
            approved: false,
            reason: reason.into(),
            reminders: vec![],
        }
    }
}

/// Run governance check. Returns a decision, and emits an `agent.start`
/// (approved) or `agent.blocked` trace event keyed by the resolved agent name.
pub fn check(root: &Path, input: &Value) -> GovernanceDecision {
    let decision = check_inner(root, input);
    let agent_name = detect_agent_name_for_trace(root, input);
    trace_log::append(
        root,
        if decision.approved {
            "agent.start"
        } else {
            "agent.blocked"
        },
        json!({"agent_id": agent_name, "reason": decision.reason}),
    );
    decision
}

fn detect_agent_name_for_trace(root: &Path, input: &Value) -> String {
    let tool_input = input.get("tool_input").cloned().unwrap_or(Value::Null);
    let description = tool_input
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let subagent_type = tool_input
        .get("subagent_type")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let name = detect_agent_name(root, description, subagent_type);
    if name.is_empty() {
        "unknown".to_string()
    } else {
        name
    }
}

fn check_inner(root: &Path, input: &Value) -> GovernanceDecision {
    let tool_input = input.get("tool_input").cloned().unwrap_or(Value::Null);
    let description = tool_input
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let prompt_text = tool_input
        .get("prompt")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let subagent_type = tool_input
        .get("subagent_type")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // Detect agent name
    let agent_name = detect_agent_name(root, description, subagent_type);
    let resolve_name = if agent_name.is_empty() {
        "unknown"
    } else {
        &agent_name
    };

    // Resolve policy
    let resolved = match policy::resolve(root, resolve_name, None) {
        Ok(r) => r,
        Err(_) => return GovernanceDecision::approve("policy_resolution_failed", vec![]),
    };

    // Check if Agent tool is blocked
    if resolved.policy.blocked_tools.contains(&"Agent".to_string()) {
        return GovernanceDecision::block(format!("Policy for {resolve_name} blocks Agent tool"));
    }

    // Check allowed_tools intersection
    if !resolved.policy.allowed_tools.is_empty()
        && !resolved.policy.allowed_tools.contains(&"Agent".to_string())
    {
        return GovernanceDecision::block(format!(
            "Policy for {resolve_name} does not include Agent in allowed_tools"
        ));
    }

    // Check blocked patterns against prompt/description content
    let content = format!("{description}\n{prompt_text}");
    for pattern in &resolved.policy.blocked_patterns {
        if regex::Regex::new(pattern).is_ok_and(|re| re.is_match(&content)) {
            return GovernanceDecision::block(format!(
                "Content matches blocked pattern: {pattern}"
            ));
        }
    }

    // Check subagent constraints
    let subagent = &resolved.policy.subagent;
    if subagent.max_concurrent == 0 {
        return GovernanceDecision::block("Policy forbids subagent dispatch (max_concurrent: 0)");
    }

    // Build reminders
    let mut reminders = Vec::new();
    if subagent.no_commit_to_main {
        reminders.push("Do NOT commit to main — verify branch first".into());
    }
    if subagent.must_verify_branch {
        reminders.push("Run `git branch --show-current` before every commit".into());
    }
    reminders.push(format!(
        "Max retries on failure: {}",
        subagent.max_retries_on_failure
    ));
    if subagent.require_commit_before_done {
        reminders.push("Must commit before reporting done".into());
    }
    if !subagent.blocked_flags.is_empty() {
        reminders.push(format!(
            "Blocked flags: {}",
            subagent.blocked_flags.join(", ")
        ));
    }

    GovernanceDecision::approve("passed all checks", reminders)
}

/// Format the decision as JSON for stdout.
pub fn format_json(decision: &GovernanceDecision) -> String {
    if decision.approved {
        json!({"decision": "approve"}).to_string()
    } else {
        json!({"decision": "block", "reason": decision.reason}).to_string()
    }
}

/// Format reminders for stderr.
pub fn format_reminders(decision: &GovernanceDecision, agent_name: &str) -> String {
    if decision.reminders.is_empty() {
        return String::new();
    }
    let lines: Vec<String> = decision
        .reminders
        .iter()
        .map(|r| format!("  - {r}"))
        .collect();
    format!(
        "[godmode:agent-governance] Policy for {agent_name}\n{}",
        lines.join("\n")
    )
}

/// Detect agent name from dispatch context.
fn detect_agent_name(root: &Path, description: &str, subagent_type: &str) -> String {
    let agents_dir = root.join("agents");

    // Check subagent_type against cfg files
    if !subagent_type.is_empty() {
        let cfg_path = agents_dir
            .join("cfg")
            .join(format!("{subagent_type}.cfg.yaml"));
        if cfg_path.exists() {
            return subagent_type.to_string();
        }
        let cfg_path2 = agents_dir
            .join("cfg")
            .join(format!("{subagent_type}-agent.cfg.yaml"));
        if cfg_path2.exists() {
            return format!("{subagent_type}-agent");
        }
    }

    // Scan description for known agent names
    if let Ok(names) = agent::list_cfg_agents(&agents_dir) {
        let desc_lower = description.to_lowercase();
        for name in &names {
            if desc_lower.contains(&name.to_lowercase()) {
                return name.clone();
            }
        }
    }

    // Common subagent types
    let types = ["Explore", "Coder", "Research"];
    for t in types {
        if subagent_type == t {
            return format!("subagent-{}", t.to_lowercase());
        }
    }

    String::new()
}
