# Rust Refactoring Patterns

## Extract Function

**Signal**: block of code used 2+ times, or block > ~15 lines with a clear purpose.

```rust
// Before
fn process(data: &[u8]) {
    let checksum = data.iter().fold(0u32, |acc, b| acc ^ *b as u32);
    println!("checksum: {checksum}");
    // ... 20 more lines ...
}

// After
fn compute_checksum(data: &[u8]) -> u32 {
    data.iter().fold(0u32, |acc, b| acc ^ *b as u32)
}
```

## Extract Module

**Signal**: file > ~300 lines, or mixed concerns (parsing + I/O + domain logic in one file).

```
src/
  parser.rs     ← extracted from lib.rs
  formatter.rs  ← extracted from lib.rs
  lib.rs        ← re-exports, orchestration only
```

## Extract Trait

**Signal**: two types share identical method sets, or you need to swap implementations.

```rust
trait Repository {
    fn find(&self, id: &str) -> Option<Item>;
    fn save(&mut self, item: Item) -> Result<()>;
}

struct SqlRepo { ... }
struct InMemoryRepo { items: Vec<Item> }

impl Repository for SqlRepo { ... }
impl Repository for InMemoryRepo { ... }
```

## Decouple I/O from Pure Logic

**Signal**: domain function calls `std::fs`, `reqwest`, or any I/O directly.

```rust
// Before (coupled)
fn load_config() -> Config {
    let raw = std::fs::read_to_string("config.toml").unwrap();
    toml::from_str(&raw).unwrap()
}

// After (decoupled — pass content in, pure logic)
fn parse_config(raw: &str) -> Result<Config> {
    toml::from_str(raw).map_err(Into::into)
}
```

## Rename

**Signal**: name misleads, contradicts usage, or is ambiguous with another identifier.

Steps:

1. Rename in one commit
2. Verify green
3. Do NOT combine with behavioural changes

## Safe Order of Operations

Never combine two patterns in one step:

1. Rename → verify green → extract
2. Extract → verify green → decouple
3. Decouple → verify green → add trait

Each step = one commit, one `cargo nextest run`.
