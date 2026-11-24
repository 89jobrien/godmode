# Rust implementation patterns: policy composition

Read this before composing policies or adding a policy to a Rust agent.

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
