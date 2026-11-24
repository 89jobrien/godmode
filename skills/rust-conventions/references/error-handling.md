# Error handling

Read this before writing a function that can fail, defining an error type, or deciding whether a call may panic.

- Use `Result<T, E>` for all recoverable errors. Never `unwrap()` or `expect()` in library
  code — return `Result` instead.
- Use the `?` operator for error propagation. Avoid `unwrap()` unless in a test or a context
  where panic is explicitly acceptable.
- Create custom error types with `thiserror`; use `anyhow` for application-level error
  aggregation.
- Use `Option<T>` for values that may legitimately be absent.
- Provide meaningful error messages with context. Error types must implement `Debug` and
  `Display` at minimum.
- Validate function arguments and return appropriate errors for invalid input.
- `panic!` is only for unrecoverable programmer errors (broken invariants). Never panic in
  response to external input.

```rust
// Prefer
fn load(path: &str) -> anyhow::Result<Config> {
    let text = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&text)?)
}

// Avoid
fn load(path: &str) -> Config {
    let text = std::fs::read_to_string(path).unwrap(); // panics on missing file
    serde_json::from_str(&text).unwrap()
}
```
