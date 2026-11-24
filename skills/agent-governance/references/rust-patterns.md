# Rust implementation patterns

Read this before building governance into a Rust agent system. These are reference implementations of the architectural approach. The hook itself is Nushell and does not use them.

The patterns below are reference implementations for building governance
into Rust agent systems. They are not used by the hook (which is Nushell)
but document the architectural approach.

### Pattern 1: Policy Struct

```rust
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
```

### Pattern 2: Policy Composition

```rust
/// Merge policies: blocked lists union, allowed lists intersect,
/// rate limits take minimum.
pub fn compose_policies(policies: &[GovernancePolicy]) -> GovernancePolicy {
    let mut combined = GovernancePolicy::default();
    for policy in policies {
        combined.blocked_tools.extend(policy.blocked_tools.clone());
        combined.blocked_patterns.extend(policy.blocked_patterns.clone());
        combined.require_human_approval.extend(
            policy.require_human_approval.clone()
        );
        combined.max_calls_per_request =
            combined.max_calls_per_request.min(policy.max_calls_per_request);
        // Intersect allowed_tools if both specify
        if !policy.allowed_tools.is_empty() {
            combined.allowed_tools = if combined.allowed_tools.is_empty() {
                policy.allowed_tools.clone()
            } else {
                combined.allowed_tools.iter()
                    .filter(|t| policy.allowed_tools.contains(t))
                    .cloned()
                    .collect()
            };
        }
    }
    combined
}
```

### Pattern 3: Governed Tool Wrapper

Wraps any tool function with policy check + rate limit + audit:

```rust
pub async fn call(&self, input: String) -> Result<String> {
    // 1. Check tool allowlist/blocklist
    match self.policy.check_tool(&self.name) {
        PolicyAction::Deny => bail!("blocked by policy"),
        PolicyAction::Review => bail!("requires human approval"),
        PolicyAction::Allow => {}
    }
    // 2. Rate limit
    let count = self.call_count.fetch_add(1, Ordering::SeqCst) + 1;
    if count > self.policy.max_calls_per_request {
        bail!("rate limit exceeded");
    }
    // 3. Content filter
    if let Some(pattern) = self.policy.check_content(&input) {
        bail!("blocked content: {pattern}");
    }
    // 4. Execute and audit
    let result = (self.inner)(input).await;
    self.audit.lock().unwrap().append(/* ... */);
    result
}
```

### Pattern 4: Trust Scoring

Track agent reliability with exponential decay:

```rust
impl TrustScore {
    pub fn current(&self, decay_rate: f64) -> f64 {
        let elapsed = now() - self.last_updated;
        self.score * (-decay_rate * elapsed).exp()
    }
}

// Gate on trust level
match trust.current(0.001) {
    t if t >= 0.7 => { /* autonomous */ }
    t if t >= 0.4 => { /* with oversight */ }
    _             => { /* deny */ }
}
```

### Pattern 5: Audit Trail

Append-only JSONL — never modify entries after write:

```rust
pub fn export_jsonl(&self, path: &str) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true).append(true).open(path)?;
    for entry in &self.entries {
        writeln!(file, "{}", serde_json::to_string(entry)?)?;
    }
    Ok(())
}
```
