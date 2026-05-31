# Common Rust Patterns

## Builder Pattern

Use when a struct has >3 optional fields:

```rust
pub struct Config {
    timeout: Duration,
    retries: u32,
    verbose: bool,
}

pub struct ConfigBuilder {
    timeout: Duration,
    retries: u32,
    verbose: bool,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self { timeout: Duration::from_secs(30), retries: 3, verbose: false }
    }
    pub fn timeout(mut self, d: Duration) -> Self { self.timeout = d; self }
    pub fn retries(mut self, n: u32) -> Self { self.retries = n; self }
    pub fn verbose(mut self) -> Self { self.verbose = true; self }
    pub fn build(self) -> Config {
        Config { timeout: self.timeout, retries: self.retries, verbose: self.verbose }
    }
}
```

## Newtype Pattern

Prevent type confusion:

```rust
pub struct UserId(pub u64);
pub struct OrderId(pub u64);
// Can't accidentally pass a UserId where OrderId is expected
```

## Graceful Degradation (Integration Pattern)

Used throughout godmode for external tool calls:

```rust
fn try_external_tool(root: &Path) -> Option<String> {
    Command::new("tool")
        .current_dir(root)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
}
```

## Feature-Gated Modules

For test-only code:

```rust
// In Cargo.toml:
// [features]
// testing = ["dep:proptest", "dep:tempfile"]

#[cfg(feature = "testing")]
pub mod testing;
```

## Error Context (anyhow)

Always add context to errors crossing boundaries:

```rust
fs::read_to_string(&path)
    .with_context(|| format!("reading {}", path.display()))?;
```
