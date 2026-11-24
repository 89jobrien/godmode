# Rust implementation patterns: governed tool wrapper

Read this before wrapping a tool call in a governance check.

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
