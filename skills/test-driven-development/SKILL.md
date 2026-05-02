---
name: "godmode:test-driven-development"
description: >
  Use when implementing any feature or bugfix, before writing implementation code. Triggers
  on "implement", "add feature", "fix bug", "write code for", or any task that produces
  production Rust code.
---

# Test-Driven Development

**Iron Law**: No production code without a prior failing test. If you wrote code before the
test, delete it and start over. No exceptions.

## Red-Green-Refactor Cycle

### 1. RED — Write a failing test

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thing_does_x() {
        // Arrange
        // Act
        // Assert — this must FAIL before you write any implementation
    }
}
```

Run it and confirm failure:

```bash
cargo nextest run -p <crate> -- <test_name>
```

**Verify the failure is for the right reason** — not a compile error, not a wrong import.
The test must exist, compile, and fail because the behavior isn't implemented yet.

### 2. GREEN — Write minimum code to pass

Write the least code that makes the test pass. No extras, no "while I'm here" additions.

```bash
cargo nextest run -p <crate>   # all tests must pass
```

### 3. REFACTOR — Clean up with tests green

> Run via `nu skills/_lib/quality-gate.nu run-quality-gate <crate>` or use the commands below:

```bash
cargo clippy -p <crate> -- -D warnings   # zero warnings required
cargo fmt -p <crate>
cargo nextest run -p <crate>             # still green after refactor
```

Commit only when clippy is clean:

1. Run: `git branch --show-current`
   Verify output matches the expected branch. Stop immediately if not.
   (`guardrails.nu check-branch <expected>`)

```bash
git commit -m "feat(<crate>): <what it does>"
```

Repeat for the next requirement.

## Rust-Specific Rules

- Tests live in `#[cfg(test)]` modules in the same file, or `tests/` for integration tests.
- Use `cargo nextest run -p <crate> -- <test_name>` to run a single test.
- `RUST_BACKTRACE=1 cargo nextest run -p <crate>` for panics.
- Trait doubles (in-memory fakes) over mocked HTTP/DB in unit tests.
- Domain code has zero imports from infrastructure crates.
- New external dependencies go behind a trait (port) — implementations in adapters.

## 3-Attempt Rule

If a test is still failing after 3 attempts, stop trying the same approach. Either:

- The architecture is wrong — step back and redesign
- The requirement is unclear — ask the user

Never brute-force past 3 failures.

## Invalid Rationalizations

| Thought                                              | Truth                                                 |
| ---------------------------------------------------- | ----------------------------------------------------- |
| "Too simple to test"                                 | Simple code breaks too                                |
| "I'll test after"                                    | Tests written after pass immediately — proves nothing |
| "I manually tested it"                               | Not repeatable, not systematic                        |
| "Just keeping code as reference while writing tests" | You will adapt it — that's test-after                 |

Any of these means you have left TDD. Delete the code and restart.

## Additional Resources

- **`references/test-patterns.md`** — unit/integration/async test patterns, naming conventions, fake vs mock
- **`helpers/test-stub.rs`** — copy-paste starting point for a new test module
