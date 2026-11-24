# Pattern 4: Lib + binary split — TQ_UNTESTED in lib

Read this before deciding how to clear a TQ_UNTESTED finding on a function in a library crate that only the binary crate exercises.

## Symptom

```text
TQ_UNTESTED  src/lib.rs:88  parse_config
TQ_UNTESTED  src/lib.rs:102  validate_schema
```

These functions are in the lib crate but have no unit tests there. Only the binary
crate exercises them through integration.

## Correct fix

Add unit tests in the lib crate. This is the real fix — these functions are testable
in isolation and should be tested there:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_config_valid_input_returns_ok() {
        let result = parse_config("key = \"value\"");
        assert!(result.is_ok());
    }
}
```

Do NOT add dispatch functions to `ignore_functions` for lib crate functions. Lib
functions can and should have unit tests.
