# Rust implementation patterns: trust scoring and audit trail

Read this before adding trust scoring or an audit trail to an agent.

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
