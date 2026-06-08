//! agent-governance — PreToolUse/Agent hook.
//! Resolves governance policy for the dispatched agent and enforces constraints.
//! Outputs JSON: {"decision": "approve"|"block", "reason": "..."}

use std::path::Path;

use serde_json::{Value, json};

use crate::agent;
use crate::policy;

/// Decision returned by the governance check.
#[derive(Debug)]
pub struct GovernanceDecision {
    pub approved: bool,
    pub reason: String,
    pub reminders: Vec<String>,
}

/// Run governance check. Returns a decision.
pub fn check(root: &Path, input: &Value) -> GovernanceDecision {
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
        Err(_) => {
            return GovernanceDecision {
                approved: true,
                reason: "policy_resolution_failed".into(),
                reminders: vec![],
            };
        }
    };

    // Check if Agent tool is blocked
    if resolved.policy.blocked_tools.contains(&"Agent".to_string()) {
        return GovernanceDecision {
            approved: false,
            reason: format!("Policy for {resolve_name} blocks Agent tool"),
            reminders: vec![],
        };
    }

    // Check allowed_tools intersection
    if !resolved.policy.allowed_tools.is_empty()
        && !resolved.policy.allowed_tools.contains(&"Agent".to_string())
    {
        return GovernanceDecision {
            approved: false,
            reason: format!("Policy for {resolve_name} does not include Agent in allowed_tools"),
            reminders: vec![],
        };
    }

    // Check blocked patterns against prompt/description content
    let content = format!("{description}\n{prompt_text}");
    for pattern in &resolved.policy.blocked_patterns {
        if regex::Regex::new(pattern).is_ok_and(|re| re.is_match(&content)) {
            return GovernanceDecision {
                approved: false,
                reason: format!("Content matches blocked pattern: {pattern}"),
                reminders: vec![],
            };
        }
    }

    // Check subagent constraints
    let subagent = &resolved.policy.subagent;
    if subagent.max_concurrent == 0 {
        return GovernanceDecision {
            approved: false,
            reason: "Policy forbids subagent dispatch (max_concurrent: 0)".into(),
            reminders: vec![],
        };
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

    GovernanceDecision {
        approved: true,
        reason: "passed all checks".into(),
        reminders,
    }
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
