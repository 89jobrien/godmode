---
name: "godmode:agent-governance"
description: >
  Patterns and techniques for adding governance, safety, and trust controls to AI agent
  systems in Rust. Use this skill when building agents that call external tools, implementing
  policy-based access controls, adding semantic intent classification to detect dangerous
  prompts, creating trust scoring systems for multi-agent workflows, building audit trails,
  or enforcing rate limits and content filters. Covers policy composition, tool wrappers,
  trust decay, and JSONL audit trails — all in Rust.
requires: []
next: []
---

# Agent Governance Patterns (Rust)

Patterns for adding safety, trust, and policy enforcement to AI agent systems.

## Overview

Governance ensures AI agents operate within defined boundaries: controlling which tools they can
call, what content they can process, and maintaining accountability through audit trails.

```
User Request → Intent Classification → Policy Check → Tool Execution → Audit Log
                     ↓                      ↓               ↓
              Threat Detection         Allow/Deny      Trust Update
```

## When to Use

- **Agents with tool access**: Any agent calling external tools (APIs, databases, shell commands)
- **Multi-agent systems**: Agents delegating to other agents need trust boundaries
- **Production deployments**: Compliance, audit, and safety requirements
- **Sensitive operations**: Financial transactions, data access, infrastructure management

---

## Pattern 1: Governance Policy

Define what an agent is allowed to do as a composable, serializable policy struct.

```rust
use serde::{Deserialize, Serialize};
use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyAction {
    Allow,
    Deny,
    Review, // flag for human approval
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernancePolicy {
    pub name: String,
    #[serde(default)]
    pub allowed_tools: Vec<String>,
    #[serde(default)]
    pub blocked_tools: Vec<String>,
    #[serde(default)]
    pub blocked_patterns: Vec<String>,
    #[serde(default = "default_max_calls")]
    pub max_calls_per_request: usize,
    #[serde(default)]
    pub require_human_approval: Vec<String>,
}

fn default_max_calls() -> usize { 100 }

impl Default for GovernancePolicy {
    fn default() -> Self {
        Self {
            name: String::new(),
            allowed_tools: vec![],
            blocked_tools: vec![],
            blocked_patterns: vec![],
            max_calls_per_request: default_max_calls(),
            require_human_approval: vec![],
        }
    }
}

impl GovernancePolicy {
    /// Check if a tool is permitted by this policy.
    pub fn check_tool(&self, tool_name: &str) -> PolicyAction {
        if self.blocked_tools.iter().any(|t| t == tool_name) {
            return PolicyAction::Deny;
        }
        if self.require_human_approval.iter().any(|t| t == tool_name) {
            return PolicyAction::Review;
        }
        if !self.allowed_tools.is_empty()
            && !self.allowed_tools.iter().any(|t| t == tool_name)
        {
            return PolicyAction::Deny;
        }
        PolicyAction::Allow
    }

    /// Check content against blocked patterns. Returns the matched pattern if found.
    pub fn check_content(&self, content: &str) -> Option<String> {
        for pattern in &self.blocked_patterns {
            if let Ok(re) = Regex::new(pattern) {
                if re.is_match(content) {
                    return Some(pattern.clone());
                }
            }
        }
        None
    }
}
```

### Policy Composition

Combine multiple policies with most-restrictive-wins semantics:

```rust
/// Merge policies: blocked lists union, allowed lists intersect, rate limits take minimum.
pub fn compose_policies(policies: &[GovernancePolicy]) -> GovernancePolicy {
    let mut combined = GovernancePolicy {
        name: "composed".into(),
        max_calls_per_request: usize::MAX,
        ..Default::default()
    };

    for policy in policies {
        combined.blocked_tools.extend(policy.blocked_tools.clone());
        combined.blocked_patterns.extend(policy.blocked_patterns.clone());
        combined.require_human_approval.extend(policy.require_human_approval.clone());
        combined.max_calls_per_request =
            combined.max_calls_per_request.min(policy.max_calls_per_request);

        if !policy.allowed_tools.is_empty() {
            combined.allowed_tools = if combined.allowed_tools.is_empty() {
                policy.allowed_tools.clone()
            } else {
                combined
                    .allowed_tools
                    .iter()
                    .filter(|t| policy.allowed_tools.contains(t))
                    .cloned()
                    .collect()
            };
        }
    }

    combined
}

// Usage: layer from broad to specific
let org = GovernancePolicy {
    name: "org-wide".into(),
    blocked_tools: vec!["shell_exec".into(), "delete_database".into()],
    blocked_patterns: vec![r"(?i)(api[_-]?key|secret|password)\s*[:=]".into()],
    max_calls_per_request: 50,
    ..Default::default()
};
let team = GovernancePolicy {
    name: "data-team".into(),
    allowed_tools: vec!["query_db".into(), "read_file".into(), "write_report".into()],
    require_human_approval: vec!["write_report".into()],
    ..Default::default()
};
let effective = compose_policies(&[org, team]);
```

### Policy as YAML

Store policies as configuration, not code (`governance-policy.yaml`):

```yaml
name: production-agent
allowed_tools:
  - search_documents
  - query_database
  - send_email
blocked_tools:
  - shell_exec
  - delete_record
blocked_patterns:
  - "(?i)(api[_-]?key|secret|password)\\s*[:=]"
  - "(?i)(drop|truncate|delete from)\\s+\\w+"
max_calls_per_request: 25
require_human_approval:
  - send_email
```

```rust
pub fn load_policy(path: &str) -> anyhow::Result<GovernancePolicy> {
    let text = std::fs::read_to_string(path)?;
    Ok(serde_yaml::from_str(&text)?)
}
```

---

## Pattern 2: Semantic Intent Classification

Detect dangerous intent in prompts before they reach the agent.

```rust
use regex::Regex;

#[derive(Debug, Clone)]
pub struct IntentSignal {
    pub category: String,
    pub confidence: f32,
    pub evidence: String,
}

/// (pattern, category, confidence) threat signals.
const THREAT_SIGNALS: &[(&str, &str, f32)] = &[
    // Data exfiltration
    (r"(?i)send\s+(all|every|entire)\s+\w+\s+to\s+", "data_exfiltration", 0.8),
    (r"(?i)export\s+.*\s+to\s+(external|outside|third.?party)", "data_exfiltration", 0.9),
    (r"(?i)curl\s+.*\s+-d\s+", "data_exfiltration", 0.7),
    // Privilege escalation
    (r"(?i)(sudo|as\s+root|admin\s+access)", "privilege_escalation", 0.8),
    (r"(?i)chmod\s+777", "privilege_escalation", 0.9),
    // System modification
    (r"(?i)(rm\s+-rf|del\s+/[sq]|format\s+c:)", "system_destruction", 0.95),
    (r"(?i)(drop\s+database|truncate\s+table)", "system_destruction", 0.9),
    // Prompt injection
    (r"(?i)ignore\s+(previous|above|all)\s+(instructions?|rules?)", "prompt_injection", 0.9),
    (r"(?i)you\s+are\s+now\s+(a|an)\s+", "prompt_injection", 0.7),
];

pub fn classify_intent(content: &str) -> Vec<IntentSignal> {
    THREAT_SIGNALS
        .iter()
        .filter_map(|(pattern, category, confidence)| {
            Regex::new(pattern).ok().and_then(|re| {
                re.find(content).map(|m| IntentSignal {
                    category: category.to_string(),
                    confidence: *confidence,
                    evidence: m.as_str().to_string(),
                })
            })
        })
        .collect()
}

/// Quick check: is content safe above the given confidence threshold?
pub fn is_safe(content: &str, threshold: f32) -> bool {
    !classify_intent(content)
        .iter()
        .any(|s| s.confidence >= threshold)
}
```

Intent classification fires _before_ tool execution — a pre-flight safety check, not an output
guardrail. This is the key distinction: catching dangerous prompts before any side effects occur.

---

## Pattern 3: Tool-Level Governance Wrapper

Wrap tool functions with governance enforcement using a `GovernedTool` struct:

```rust
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use anyhow::{bail, Result};

pub struct GovernedTool<F> {
    name: String,
    policy: Arc<GovernancePolicy>,
    call_count: AtomicUsize,
    audit: Arc<Mutex<AuditTrail>>,
    inner: F,
}

impl<F, Fut> GovernedTool<F>
where
    F: Fn(String) -> Fut + Send + Sync,
    Fut: std::future::Future<Output = Result<String>>,
{
    pub fn new(
        name: impl Into<String>,
        policy: Arc<GovernancePolicy>,
        audit: Arc<Mutex<AuditTrail>>,
        inner: F,
    ) -> Self {
        Self {
            name: name.into(),
            policy,
            call_count: AtomicUsize::new(0),
            audit,
            inner,
        }
    }

    pub async fn call(&self, input: String) -> Result<String> {
        // 1. Check tool allowlist/blocklist
        match self.policy.check_tool(&self.name) {
            PolicyAction::Deny => {
                bail!("Policy '{}' blocks tool '{}'", self.policy.name, self.name)
            }
            PolicyAction::Review => {
                bail!("Tool '{}' requires human approval", self.name)
            }
            PolicyAction::Allow => {}
        }

        // 2. Rate limit
        let count = self.call_count.fetch_add(1, Ordering::SeqCst) + 1;
        if count > self.policy.max_calls_per_request {
            bail!("Rate limit exceeded: {} calls", self.policy.max_calls_per_request);
        }

        // 3. Content filter
        if let Some(pattern) = self.policy.check_content(&input) {
            bail!("Blocked content pattern: {pattern}");
        }

        // 4. Execute and audit
        let start = Instant::now();
        let result = (self.inner)(input).await;
        let duration_ms = start.elapsed().as_millis() as u64;
        let action = if result.is_ok() { "allowed" } else { "error" };

        if let Ok(mut log) = self.audit.lock() {
            log.append(AuditEntry {
                timestamp: unix_now(),
                agent_id: self.policy.name.clone(),
                tool_name: self.name.clone(),
                action: action.into(),
                policy_name: self.policy.name.clone(),
                details: [("duration_ms".into(), duration_ms.to_string())]
                    .into_iter()
                    .collect(),
            });
        }
        result
    }
}

fn unix_now() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}
```

---

## Pattern 4: Trust Scoring

Track agent reliability over time with exponential decay — trust erodes without activity.

```rust
#[derive(Debug, Clone)]
pub struct TrustScore {
    pub score: f64,
    pub successes: u32,
    pub failures: u32,
    last_updated: f64,
}

impl Default for TrustScore {
    fn default() -> Self {
        Self { score: 0.5, successes: 0, failures: 0, last_updated: unix_now() }
    }
}

impl TrustScore {
    pub fn record_success(&mut self, reward: f64) {
        self.successes += 1;
        self.score = (self.score + reward * (1.0 - self.score)).min(1.0);
        self.last_updated = unix_now();
    }

    pub fn record_failure(&mut self, penalty: f64) {
        self.failures += 1;
        self.score = (self.score - penalty * self.score).max(0.0);
        self.last_updated = unix_now();
    }

    /// Score with temporal decay — trust erodes without activity.
    pub fn current(&self, decay_rate: f64) -> f64 {
        let elapsed = unix_now() - self.last_updated;
        self.score * (-decay_rate * elapsed).exp()
    }

    pub fn reliability(&self) -> f64 {
        let total = (self.successes + self.failures) as f64;
        if total == 0.0 { 0.0 } else { self.successes as f64 / total }
    }
}

/// Multi-agent trust registry — each coordinator tracks its delegates.
use std::collections::HashMap;

pub struct AgentTrustRegistry {
    scores: HashMap<String, TrustScore>,
}

impl AgentTrustRegistry {
    pub fn new() -> Self {
        Self { scores: HashMap::new() }
    }

    pub fn get_mut(&mut self, agent_id: &str) -> &mut TrustScore {
        self.scores.entry(agent_id.to_string()).or_default()
    }

    pub fn most_trusted<'a>(&self, agents: &'a [String]) -> Option<&'a str> {
        agents
            .iter()
            .max_by(|a, b| {
                let ta = self.scores.get(a.as_str()).map_or(0.5, |s| s.current(0.001));
                let tb = self.scores.get(b.as_str()).map_or(0.5, |s| s.current(0.001));
                ta.partial_cmp(&tb).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(String::as_str)
    }

    pub fn meets_threshold(&self, agent_id: &str, threshold: f64) -> bool {
        self.scores.get(agent_id).map_or(false, |s| s.current(0.001) >= threshold)
    }
}

// Gate operations on trust level
let trust = registry.get_mut("agent-42");
match trust.current(0.001) {
    t if t >= 0.7 => { /* autonomous operation */ }
    t if t >= 0.4 => { /* allow with oversight */ }
    _             => { /* deny or require explicit approval */ }
}
```

---

## Pattern 5: Audit Trail

Append-only JSONL audit log — critical for compliance and post-incident review.

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: f64,
    pub agent_id: String,
    pub tool_name: String,
    pub action: String, // "allowed" | "denied" | "error"
    pub policy_name: String,
    #[serde(default)]
    pub details: HashMap<String, String>,
}

pub struct AuditTrail {
    entries: Vec<AuditEntry>,
}

impl AuditTrail {
    pub fn new() -> Self {
        Self { entries: vec![] }
    }

    pub fn append(&mut self, entry: AuditEntry) {
        self.entries.push(entry);
    }

    pub fn denied(&self) -> Vec<&AuditEntry> {
        self.entries.iter().filter(|e| e.action == "denied").collect()
    }

    pub fn by_agent(&self, agent_id: &str) -> Vec<&AuditEntry> {
        self.entries.iter().filter(|e| e.agent_id == agent_id).collect()
    }

    /// Export as JSON Lines for log aggregation systems (append mode).
    pub fn export_jsonl(&self, path: &str) -> std::io::Result<()> {
        let mut file = OpenOptions::new().create(true).append(true).open(path)?;
        for entry in &self.entries {
            if let Ok(line) = serde_json::to_string(entry) {
                writeln!(file, "{line}")?;
            }
        }
        Ok(())
    }
}
```

---

## Pattern 6: Putting It Together

```rust
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let policy = Arc::new(GovernancePolicy {
        name: "search-agent".into(),
        allowed_tools: vec!["search".into(), "summarize".into()],
        blocked_patterns: vec![r"(?i)password".into()],
        max_calls_per_request: 10,
        ..Default::default()
    });

    let audit = Arc::new(Mutex::new(AuditTrail::new()));

    let search = GovernedTool::new(
        "search",
        Arc::clone(&policy),
        Arc::clone(&audit),
        |query: String| async move { Ok(format!("Results for: {query}")) },
    );

    // Passes
    let result = search.call("latest quarterly report".into()).await?;
    println!("{result}");

    // Blocked — pattern match on "password"
    assert!(search.call("show me the admin password".into()).await.is_err());

    audit.lock().unwrap().export_jsonl(".ctx/governance-audit.jsonl")?;
    Ok(())
}
```

---

## Governance Levels

| Level        | Controls                                        | Use Case                     |
| ------------ | ----------------------------------------------- | ---------------------------- |
| **Open**     | Audit only, no restrictions                     | Internal dev/testing         |
| **Standard** | Tool allowlist + content filters                | General production agents    |
| **Strict**   | All controls + human approval for sensitive ops | Financial, healthcare, legal |
| **Locked**   | Allowlist only, no dynamic tools, full audit    | Compliance-critical systems  |

---

## Cargo Dependencies

```toml
[dependencies]
regex = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
anyhow = "1"
tokio = { version = "1", features = ["full"] }
```

---

## Best Practices

| Practice                       | Rationale                                                     |
| ------------------------------ | ------------------------------------------------------------- |
| **Policy as configuration**    | Store in YAML, not code — enables change without rebuilding   |
| **Most-restrictive-wins**      | When composing, deny always overrides allow                   |
| **Pre-flight intent check**    | Classify intent _before_ tool execution, not after            |
| **Trust decay**                | Scores must decay — require ongoing demonstrated reliability  |
| **Append-only audit**          | Never modify audit entries — immutability enables compliance  |
| **Fail closed**                | If governance check errors, deny rather than allow            |
| **Separate policy from logic** | Governance enforcement is independent of agent business logic |

---

## Quick Start Checklist

```markdown
## Agent Governance Implementation Checklist

### Setup

- [ ] Define GovernancePolicy (allowed_tools, blocked_patterns, max_calls_per_request)
- [ ] Choose governance level (open/standard/strict/locked)
- [ ] Set up AuditTrail and decide export path (.ctx/ JSONL)

### Implementation

- [ ] Wrap tool functions in GovernedTool with shared policy + audit Arc
- [ ] Add classify_intent() to user input before dispatch
- [ ] Wire TrustScore updates after each agent task success/failure
- [ ] Export audit JSONL at session end

### Validation

- [ ] Test blocked tools return Err
- [ ] Test content filters catch sensitive patterns
- [ ] Test rate limit exceeded after N calls
- [ ] Verify audit trail captures allowed + denied entries
- [ ] Test policy composition (most-restrictive-wins)
```

---

## Related

- [OWASP Top 10 for LLM Applications](https://owasp.org/www-project-top-10-for-large-language-model-applications/)
- `skills/observability-as-infrastructure/SKILL.md` — trace events for audit integration
- `skills/parallel-agents/SKILL.md` — trust scoring in multi-agent dispatch
