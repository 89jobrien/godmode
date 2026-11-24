# Rust implementation patterns

Read this before building governance into a Rust agent system. This file holds Pattern 1 only. Patterns 2 to 5 are in the `rust-patterns-*.md` files listed in SKILL.md. These are reference implementations of the architectural approach. The hook itself is Nushell and does not use them.

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
