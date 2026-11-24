# Code style and testing

Read this before committing Rust code, writing doc comments, or adding tests: formatting and lint gates, line length, doc comments, test layout, and the example test shape.

## Code Style

- Run `rustfmt` (`taskit check fmt`) before every commit.
- Run `taskit check lint` and fix all warnings.
- Keep lines at or under 100 characters.
- Place doc comments (`///`) immediately above the item they document.
- Use `//!` for module-level documentation.
- Document error conditions, panic scenarios, and safety considerations.

---

## Testing

- Write unit tests in `#[cfg(test)]` modules in the same file as the code under test.
- Write integration tests in `tests/` with descriptive file names.
- Use `taskit test run` (preferred over `taskit test run`) for test filtering and parallelism.
- Test edge cases explicitly — not just the happy path.
- Examples in doc comments must compile and use `?`, not `unwrap()`.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_tool_denies_blocked() {
        let policy = GovernancePolicy {
            blocked_tools: vec!["shell_exec".into()],
            ..Default::default()
        };
        assert_eq!(policy.check_tool("shell_exec"), PolicyAction::Deny);
    }
}
```
