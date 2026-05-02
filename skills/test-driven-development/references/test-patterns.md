# Rust Test Patterns

## Unit Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Arrange-Act-Assert
    #[test]
    fn <thing>_<condition>_<expected_outcome>() {
        // Arrange
        let input = ...;

        // Act
        let result = function_under_test(input);

        // Assert
        assert_eq!(result, expected);
    }
}
```

## Integration Tests (`tests/` dir)

```rust
// tests/my_integration.rs
use my_crate::SomeType;

#[test]
fn end_to_end_scenario() {
    // Uses the public API only — no internal module access
}
```

## In-Memory Fakes (preferred over mocks)

```rust
struct FakeRepo {
    items: Vec<Item>,
}

impl ItemRepository for FakeRepo {
    fn find(&self, id: &str) -> Option<&Item> {
        self.items.iter().find(|i| i.id == id)
    }
}
```

Fakes are real implementations. They implement the trait. They do not use mock frameworks.

## Error Case Coverage

Every `Result`-returning function needs at least one error test:

```rust
#[test]
fn returns_err_when_input_is_invalid() {
    let result = parse_thing("not valid");
    assert!(result.is_err());
}
```

## Async Tests

```rust
#[tokio::test]
async fn async_thing_works() {
    let result = async_fn().await;
    assert!(result.is_ok());
}
```

## Naming Convention

`<unit>_<scenario>_<expected>` — readable as a sentence:

- `parser_empty_input_returns_err`
- `graph_all_deps_done_unlocks_task`
- `run_cmd_with_pipe_uses_shell`

## Running Specific Tests

```bash
cargo nextest run -p <crate> -- <test_name>          # exact match
cargo nextest run -p <crate> -- <prefix>             # prefix match
RUST_BACKTRACE=1 cargo nextest run -p <crate>        # with backtrace
```
