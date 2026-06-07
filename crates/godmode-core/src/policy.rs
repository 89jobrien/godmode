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
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use crate::agent;

// ── Types ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyAction {
    Allow,
    Deny,
    Review,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceLevel {
    Open,
    Standard,
    Strict,
    Locked,
}

impl std::fmt::Display for GovernanceLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open => write!(f, "open"),
            Self::Standard => write!(f, "standard"),
            Self::Strict => write!(f, "strict"),
            Self::Locked => write!(f, "locked"),
        }
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct SubagentConstraints {
    pub max_concurrent: usize,
    pub must_verify_branch: bool,
    pub no_commit_to_main: bool,
    pub max_retries_on_failure: usize,
    pub require_commit_before_done: bool,
    pub blocked_flags: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AuditSettings {
    pub enabled: bool,
    pub format: String,
    pub path: String,
    pub log_allowed: bool,
    pub log_denied: bool,
    pub log_reviews: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct GovernancePolicy {
    pub name: String,
    #[serde(default)]
    pub level: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub inherits: String,
    #[serde(default)]
    pub allowed_tools: Vec<String>,
    #[serde(default)]
    pub blocked_tools: Vec<String>,
    #[serde(default)]
    pub blocked_patterns: Vec<String>,
    #[serde(default = "default_max_calls")]
    pub max_calls_per_dispatch: usize,
    #[serde(default)]
    pub require_human_approval: Vec<String>,
    #[serde(default)]
    pub subagent: SubagentConstraints,
    #[serde(default)]
    pub audit: AuditSettings,
}

fn default_max_calls() -> usize {
    200
}

/// Result of resolving a policy for an agent.
#[derive(Debug, Clone, Serialize)]
pub struct ResolvedPolicy {
    pub policy: GovernancePolicy,
    pub agent: String,
    pub category: String,
    pub level: String,
    pub sources: Vec<String>,
}

/// Result of checking a tool call against a policy.
#[derive(Debug, Clone, Serialize)]
pub struct CheckResult {
    pub action: PolicyAction,
    pub reason: String,
}

// ── Loading ──────────────────────────────────────────────────────────

fn policies_dir(root: &Path) -> PathBuf {
    root.join("skills")
        .join("agent-governance")
        .join("policies")
}

fn load_policy(path: &Path) -> Result<GovernancePolicy> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("reading policy {}", path.display()))?;
    serde_yaml::from_str(&raw).with_context(|| format!("parsing policy YAML {}", path.display()))
}

/// Look up the category for an agent from `agents/cfg/<name>.cfg.yaml`.
///
/// Searches by filename first, then falls back to scanning all configs
/// and matching by the `name` field inside the YAML (agent configs often
/// use a different filename than their internal name).
pub fn agent_category(root: &Path, agent_name: &str) -> Option<String> {
    let cfg_dir = root.join("agents").join("cfg");

    // 1. Try filename-based lookup
    for candidate in [
        cfg_dir.join(format!("{agent_name}.cfg.yaml")),
        cfg_dir.join(format!("{agent_name}-agent.cfg.yaml")),
    ] {
        if candidate.exists()
            && let Ok(def) = agent::load(&candidate)
            && !def.category.is_empty()
        {
            return Some(def.category);
        }
    }

    // 2. Fall back to scanning all configs by internal name field
    if let Ok(entries) = std::fs::read_dir(&cfg_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "yaml")
                && let Ok(def) = agent::load(&path)
                && def.name == agent_name
                && !def.category.is_empty()
            {
                return Some(def.category);
            }
        }
    }

    None
}

// ── Composition ──────────────────────────────────────────────────────

/// How to apply an overlay's allowed_tools.
#[derive(Clone, Copy)]
enum AllowedToolsMode {
    /// Category overlays replace allowed_tools (specialization).
    Replace,
    /// Level overlays intersect allowed_tools (restriction).
    Intersect,
}

/// Compose an overlay on top of a base policy.
///
/// `mode` controls how allowed_tools are merged:
/// - `Replace`: overlay replaces base (for category specialization)
/// - `Intersect`: overlay intersects with base (for level restriction)
///
/// All other fields use most-restrictive-wins: blocked lists union,
/// rate limits take minimum, human-approval unions.
fn compose(base: &mut GovernancePolicy, overlay: &GovernancePolicy, mode: AllowedToolsMode) {
    // Union blocked_tools
    for tool in &overlay.blocked_tools {
        if !base.blocked_tools.contains(tool) {
            base.blocked_tools.push(tool.clone());
        }
    }

    // Union blocked_patterns
    for pat in &overlay.blocked_patterns {
        if !base.blocked_patterns.contains(pat) {
            base.blocked_patterns.push(pat.clone());
        }
    }

    // Union require_human_approval
    for item in &overlay.require_human_approval {
        if !base.require_human_approval.contains(item) {
            base.require_human_approval.push(item.clone());
        }
    }

    // Merge allowed_tools based on mode
    if !overlay.allowed_tools.is_empty() {
        match mode {
            AllowedToolsMode::Replace => {
                base.allowed_tools.clone_from(&overlay.allowed_tools);
            }
            AllowedToolsMode::Intersect => {
                if base.allowed_tools.is_empty() {
                    base.allowed_tools.clone_from(&overlay.allowed_tools);
                } else {
                    base.allowed_tools
                        .retain(|t| overlay.allowed_tools.contains(t));
                }
            }
        }
    }

    // Min max_calls_per_dispatch
    base.max_calls_per_dispatch = base
        .max_calls_per_dispatch
        .min(overlay.max_calls_per_dispatch);

    // Subagent: per-field most-restrictive
    base.subagent.max_concurrent = base
        .subagent
        .max_concurrent
        .min(overlay.subagent.max_concurrent);
    base.subagent.must_verify_branch |= overlay.subagent.must_verify_branch;
    base.subagent.no_commit_to_main |= overlay.subagent.no_commit_to_main;
    base.subagent.max_retries_on_failure = base
        .subagent
        .max_retries_on_failure
        .min(overlay.subagent.max_retries_on_failure);
    base.subagent.require_commit_before_done |= overlay.subagent.require_commit_before_done;
    for flag in &overlay.subagent.blocked_flags {
        if !base.subagent.blocked_flags.contains(flag) {
            base.subagent.blocked_flags.push(flag.clone());
        }
    }
}

// ── Public API ───────────────────────────────────────────────────────

/// Resolve the effective governance policy for an agent.
///
/// Loads `default.yaml`, overlays the category policy (looked up from
/// `agents/cfg/`), then overlays the governance level.
pub fn resolve(
    root: &Path,
    agent_name: &str,
    level_override: Option<&GovernanceLevel>,
) -> Result<ResolvedPolicy> {
    let dir = policies_dir(root);
    let default_path = dir.join("default.yaml");

    if !default_path.exists() {
        bail!("no default governance policy at {}", default_path.display());
    }

    let mut policy = load_policy(&default_path)?;
    let mut sources = vec!["default.yaml".to_string()];

    // Category overlay
    let category = agent_category(root, agent_name).unwrap_or_default();
    if !category.is_empty() {
        let cat_path = dir.join("by-category").join(format!("{category}.yaml"));
        if cat_path.exists() {
            let cat_policy = load_policy(&cat_path)?;
            compose(&mut policy, &cat_policy, AllowedToolsMode::Replace);
            sources.push(format!("by-category/{category}.yaml"));
        }
    }

    // Level overlay
    let effective_level = match level_override {
        Some(l) => l.to_string(),
        None => {
            if policy.level.is_empty() {
                "standard".to_string()
            } else {
                policy.level.clone()
            }
        }
    };
    let level_path = dir.join("levels").join(format!("{effective_level}.yaml"));
    if level_path.exists() {
        let level_policy = load_policy(&level_path)?;
        compose(&mut policy, &level_policy, AllowedToolsMode::Intersect);
        sources.push(format!("levels/{effective_level}.yaml"));
    }

    Ok(ResolvedPolicy {
        policy,
        agent: agent_name.to_string(),
        category,
        level: effective_level,
        sources,
    })
}

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
                    // Bad pattern — log but don't block
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

/// List all available policies (default + categories + levels).
pub fn list_policies(root: &Path) -> Result<PolicyIndex> {
    let dir = policies_dir(root);
    let mut index = PolicyIndex {
        default: None,
        categories: HashMap::new(),
        levels: HashMap::new(),
    };

    let default_path = dir.join("default.yaml");
    if default_path.exists() {
        index.default = Some(load_policy(&default_path)?);
    }

    let cat_dir = dir.join("by-category");
    if cat_dir.is_dir()
        && let Ok(entries) = std::fs::read_dir(&cat_dir)
    {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "yaml") {
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();
                if let Ok(p) = load_policy(&path) {
                    index.categories.insert(name, p);
                }
            }
        }
    }

    let level_dir = dir.join("levels");
    if level_dir.is_dir()
        && let Ok(entries) = std::fs::read_dir(&level_dir)
    {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "yaml") {
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();
                if let Ok(p) = load_policy(&path) {
                    index.levels.insert(name, p);
                }
            }
        }
    }

    Ok(index)
}

#[derive(Debug, Serialize)]
pub struct PolicyIndex {
    pub default: Option<GovernancePolicy>,
    pub categories: HashMap<String, GovernancePolicy>,
    pub levels: HashMap<String, GovernancePolicy>,
}

// ── Audit ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceEvent {
    pub ts: String,
    pub event: String,
    pub action: String,
    pub agent_id: String,
    pub tool_name: String,
    pub reason: String,
    #[serde(default)]
    pub pattern: String,
    #[serde(default)]
    pub session_id: String,
}

/// Append a governance audit event to the JSONL trail.
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_policy(dir: &Path, rel: &str, content: &str) {
        let path = dir.join(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, content).unwrap();
    }

    fn setup_policies(tmp: &Path) {
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

        // Agent config for test
        let cfg_dir = tmp.join("agents").join("cfg");
        std::fs::create_dir_all(&cfg_dir).unwrap();
        std::fs::write(
            cfg_dir.join("test-planner.cfg.yaml"),
            "name: test-planner\ncategory: plan\n",
        )
        .unwrap();
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
        // Intersection: default has Bash, plan does not → no Bash
        assert!(!resolved.policy.allowed_tools.contains(&"Bash".to_string()));
        // Union blocked: plan blocks Bash and Agent
        assert!(resolved.policy.blocked_tools.contains(&"Bash".to_string()));
        assert!(resolved.policy.blocked_tools.contains(&"Agent".to_string()));
        // Min rate limit: min(200, 150) = 150
        assert_eq!(resolved.policy.max_calls_per_dispatch, 150);
    }

    #[test]
    fn resolve_with_level_override() {
        let tmp = TempDir::new().unwrap();
        setup_policies(tmp.path());
        let resolved =
            resolve(tmp.path(), "unknown-agent", Some(&GovernanceLevel::Strict)).unwrap();
        assert_eq!(resolved.level, "strict");
        // Strict narrows allowed_tools to Read, Glob, Grep
        assert!(!resolved.policy.allowed_tools.contains(&"Bash".to_string()));
        assert!(resolved.policy.allowed_tools.contains(&"Read".to_string()));
        // Strict requires human approval for Write
        assert!(
            resolved
                .policy
                .require_human_approval
                .contains(&"Write".to_string())
        );
        // min(200, 50) = 50
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
