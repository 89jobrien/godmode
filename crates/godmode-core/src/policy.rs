//! Governance policy engine — loads, composes, and enforces agent policies.
//!
//! Policies live in `skills/agent-governance/policies/`:
//!   - `default.yaml` — baseline all agents inherit
//!   - `by-category/<cat>.yaml` — per-category overrides
//!   - `levels/<level>.yaml` — governance level overlays
//!
//! Composition follows most-restrictive-wins: blocked lists union,
//! allowed lists intersect, rate limits take minimum.

use std::collections::HashMap;

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

#[path = "policy_engine.rs"]
/// Tool-call enforcement and governance audit event storage.
pub mod policy_engine;
#[path = "policy_loader.rs"]
/// Policy discovery, loading, and overlay composition.
pub mod policy_loader;

pub use policy_engine::{GovernanceEvent, check_tool, emit_audit_event, read_audit_events};
pub use policy_loader::{agent_category, list_policies, resolve};

// ── Types ────────────────────────────────────────────────────────────

/// Decision produced when a policy evaluates a tool call.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyAction {
    /// Permit the tool call without additional approval.
    Allow,
    /// Reject the tool call.
    Deny,
    /// Require human review before proceeding.
    Review,
}

/// Named strictness level used to select a policy overlay.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceLevel {
    /// Apply the least restrictive available overlay.
    Open,
    /// Apply the normal governance overlay.
    Standard,
    /// Apply additional restrictions and approval requirements.
    Strict,
    /// Apply the most restrictive available overlay.
    Locked,
}

impl GovernanceLevel {
    /// Return the lowercase configuration name for this level.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Standard => "standard",
            Self::Strict => "strict",
            Self::Locked => "locked",
        }
    }
}

impl std::fmt::Display for GovernanceLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for GovernanceLevel {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "open" => Ok(Self::Open),
            "standard" => Ok(Self::Standard),
            "strict" => Ok(Self::Strict),
            "locked" => Ok(Self::Locked),
            other => bail!("unknown governance level: {other}"),
        }
    }
}

/// Limits applied to subagents launched under a policy.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct SubagentConstraints {
    /// Maximum number of subagents that may run concurrently.
    pub max_concurrent: usize,
    /// Whether a subagent must verify its current branch before committing.
    pub must_verify_branch: bool,
    /// Whether committing directly to the main branch is prohibited.
    pub no_commit_to_main: bool,
    /// Maximum retry count after a failed operation.
    pub max_retries_on_failure: usize,
    /// Whether completion requires at least one commit.
    pub require_commit_before_done: bool,
    /// Command-line flags that subagents may not use.
    pub blocked_flags: Vec<String>,
}

/// Controls for governance audit trail emission.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AuditSettings {
    /// Whether governance audit logging is enabled.
    pub enabled: bool,
    /// Serialization format used for audit records.
    pub format: String,
    /// Path where audit records are written.
    pub path: String,
    /// Whether allowed decisions are logged.
    pub log_allowed: bool,
    /// Whether denied decisions are logged.
    pub log_denied: bool,
    /// Whether decisions requiring review are logged.
    pub log_reviews: bool,
}

/// Deserialized governance rules and operational limits.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct GovernancePolicy {
    /// Human-readable policy name.
    pub name: String,
    #[serde(default)]
    /// Governance level associated with this policy.
    pub level: String,
    #[serde(default)]
    /// Agent category targeted by this policy.
    pub category: String,
    #[serde(default)]
    /// Name of the policy this policy declares as its parent.
    pub inherits: String,
    #[serde(default)]
    /// Tools that may be called; an empty list imposes no allowlist.
    pub allowed_tools: Vec<String>,
    #[serde(default)]
    /// Tools that are always denied.
    pub blocked_tools: Vec<String>,
    #[serde(default)]
    /// Regular expressions that deny matching tool input.
    pub blocked_patterns: Vec<String>,
    #[serde(default = "default_max_calls")]
    /// Maximum tool calls permitted in one dispatch.
    pub max_calls_per_dispatch: usize,
    #[serde(default)]
    /// Tools that require human approval before use.
    pub require_human_approval: Vec<String>,
    #[serde(default)]
    /// Constraints inherited by launched subagents.
    pub subagent: SubagentConstraints,
    #[serde(default)]
    /// Audit logging settings for policy decisions.
    pub audit: AuditSettings,
}

/// Default maximum tool calls allowed in one dispatch.
pub(crate) fn default_max_calls() -> usize {
    200
}

/// Result of resolving a policy for an agent.
#[derive(Debug, Clone, Serialize)]
pub struct ResolvedPolicy {
    /// Effective composed policy.
    pub policy: GovernancePolicy,
    /// Agent for which the policy was resolved.
    pub agent: String,
    /// Detected agent category, or an empty string when unknown.
    pub category: String,
    /// Effective governance level.
    pub level: String,
    /// Relative policy files composed to produce the result.
    pub sources: Vec<String>,
}

/// Result of checking a tool call against a policy.
#[derive(Debug, Clone, Serialize)]
pub struct CheckResult {
    /// Enforcement action selected by the policy.
    pub action: PolicyAction,
    /// Human-readable explanation of the decision.
    pub reason: String,
}

/// Collection of all policy files discovered under a project root.
#[derive(Debug, Serialize)]
pub struct PolicyIndex {
    /// Baseline policy, when present.
    pub default: Option<GovernancePolicy>,
    /// Policies keyed by agent category.
    pub categories: HashMap<String, GovernancePolicy>,
    /// Policies keyed by governance level.
    pub levels: HashMap<String, GovernancePolicy>,
}

/// How to apply an overlay's allowed_tools.
#[derive(Clone, Copy)]
pub(crate) enum AllowedToolsMode {
    /// Category overlays replace allowed_tools (specialization).
    Replace,
    /// Level overlays intersect allowed_tools (restriction).
    Intersect,
}

#[cfg(test)]
mod tests {
    use super::policy_engine::GovernanceEvent;
    use super::*;
    use tempfile::TempDir;

    fn write_policy(dir: &std::path::Path, rel: &str, content: &str) {
        let path = dir.join(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, content).unwrap();
    }

    fn setup_policies(tmp: &std::path::Path) {
        let policies = tmp.join("skills").join("agent-governance").join("policies");

        write_policy(
            &policies,
            "default.yaml",
            r#"
name: default
level: standard
allowed_tools: [Read, Write, Edit, Bash, Glob, Grep]
blocked_tools: []
blocked_patterns:
  - "(?i)--no-verify"
max_calls_per_dispatch: 200
require_human_approval: []
subagent:
  max_concurrent: 5
  must_verify_branch: true
  no_commit_to_main: true
  max_retries_on_failure: 3
  require_commit_before_done: true
  blocked_flags: ["--no-verify"]
"#,
        );

        write_policy(
            &policies,
            "by-category/plan.yaml",
            r#"
name: category-plan
category: plan
allowed_tools: [Read, Write, Glob, Grep]
blocked_tools: [Bash, Agent]
blocked_patterns:
  - "(?i)cargo\\s+(build|test|run)"
max_calls_per_dispatch: 150
"#,
        );

        write_policy(
            &policies,
            "levels/strict.yaml",
            r#"
name: level-strict
level: strict
allowed_tools: [Read, Glob, Grep]
blocked_tools: [Agent]
max_calls_per_dispatch: 50
require_human_approval: [Write, Edit, Bash]
subagent:
  max_concurrent: 2
  must_verify_branch: true
  no_commit_to_main: true
  max_retries_on_failure: 1
  require_commit_before_done: true
"#,
        );

        let cfg_dir = tmp.join("agents").join("cfg");
        std::fs::create_dir_all(&cfg_dir).unwrap();
        std::fs::write(
            cfg_dir.join("test-planner.cfg.yaml"),
            "name: test-planner\ncategory: plan\n",
        )
        .unwrap();
    }

    #[test]
    fn default_max_calls_is_stable() {
        assert_eq!(default_max_calls(), 200);
    }

    #[test]
    fn resolve_default_policy() {
        let tmp = TempDir::new().unwrap();
        setup_policies(tmp.path());
        let resolved = resolve(tmp.path(), "unknown-agent", None).unwrap();
        assert_eq!(resolved.level, "standard");
        assert_eq!(resolved.sources, vec!["default.yaml"]);
        assert!(resolved.policy.allowed_tools.contains(&"Bash".to_string()));
    }

    #[test]
    fn resolve_with_category() {
        let tmp = TempDir::new().unwrap();
        setup_policies(tmp.path());
        let resolved = resolve(tmp.path(), "test-planner", None).unwrap();
        assert_eq!(resolved.category, "plan");
        assert!(
            resolved
                .sources
                .contains(&"by-category/plan.yaml".to_string())
        );
        assert!(!resolved.policy.allowed_tools.contains(&"Bash".to_string()));
        assert!(resolved.policy.blocked_tools.contains(&"Bash".to_string()));
        assert!(resolved.policy.blocked_tools.contains(&"Agent".to_string()));
        assert_eq!(resolved.policy.max_calls_per_dispatch, 150);
    }

    #[test]
    fn resolve_with_level_override() {
        let tmp = TempDir::new().unwrap();
        setup_policies(tmp.path());
        let resolved =
            resolve(tmp.path(), "unknown-agent", Some(&GovernanceLevel::Strict)).unwrap();
        assert_eq!(resolved.level, "strict");
        assert!(!resolved.policy.allowed_tools.contains(&"Bash".to_string()));
        assert!(resolved.policy.allowed_tools.contains(&"Read".to_string()));
        assert!(
            resolved
                .policy
                .require_human_approval
                .contains(&"Write".to_string())
        );
        assert_eq!(resolved.policy.max_calls_per_dispatch, 50);
    }

    #[test]
    fn check_tool_blocked() {
        let policy = GovernancePolicy {
            blocked_tools: vec!["Agent".to_string()],
            ..Default::default()
        };
        let result = check_tool(&policy, "Agent", None);
        assert_eq!(result.action, PolicyAction::Deny);
    }

    #[test]
    fn check_tool_not_in_allowed() {
        let policy = GovernancePolicy {
            allowed_tools: vec!["Read".to_string(), "Grep".to_string()],
            ..Default::default()
        };
        let result = check_tool(&policy, "Bash", None);
        assert_eq!(result.action, PolicyAction::Deny);
    }

    #[test]
    fn check_tool_requires_approval() {
        let policy = GovernancePolicy {
            require_human_approval: vec!["Write".to_string()],
            ..Default::default()
        };
        let result = check_tool(&policy, "Write", None);
        assert_eq!(result.action, PolicyAction::Review);
    }

    #[test]
    fn check_tool_wildcard_approval() {
        let policy = GovernancePolicy {
            require_human_approval: vec!["*".to_string()],
            ..Default::default()
        };
        let result = check_tool(&policy, "Bash", None);
        assert_eq!(result.action, PolicyAction::Review);
    }

    #[test]
    fn check_tool_content_pattern() {
        let policy = GovernancePolicy {
            blocked_patterns: vec!["(?i)--no-verify".to_string()],
            ..Default::default()
        };
        let result = check_tool(&policy, "Bash", Some("git commit --no-verify -m 'test'"));
        assert_eq!(result.action, PolicyAction::Deny);
        assert!(result.reason.contains("--no-verify"));
    }

    #[test]
    fn check_tool_allows_clean_content() {
        let policy = GovernancePolicy {
            allowed_tools: vec!["Bash".to_string()],
            blocked_patterns: vec!["(?i)--no-verify".to_string()],
            ..Default::default()
        };
        let result = check_tool(&policy, "Bash", Some("git commit -m 'test'"));
        assert_eq!(result.action, PolicyAction::Allow);
    }

    #[test]
    fn list_policies_finds_all() {
        let tmp = TempDir::new().unwrap();
        setup_policies(tmp.path());
        let index = list_policies(tmp.path()).unwrap();
        assert!(index.default.is_some());
        assert!(index.categories.contains_key("plan"));
        assert!(index.levels.contains_key("strict"));
    }

    #[test]
    fn audit_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let event = GovernanceEvent {
            ts: "2026-06-04T14:00:00+0000".to_string(),
            event: "governance.check".to_string(),
            action: "denied".to_string(),
            agent_id: "test-agent".to_string(),
            tool_name: "Bash".to_string(),
            reason: "blocked pattern".to_string(),
            pattern: "(?i)--no-verify".to_string(),
            session_id: "abc123".to_string(),
        };
        emit_audit_event(tmp.path(), &event).unwrap();
        let events = read_audit_events(tmp.path(), Some("2026-06-04")).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].agent_id, "test-agent");

        let empty = read_audit_events(tmp.path(), Some("2025-01-01")).unwrap();
        assert!(empty.is_empty());
    }
}
