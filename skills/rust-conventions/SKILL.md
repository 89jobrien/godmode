---
name: "godmode:rust-conventions"
description: >
  Rust coding conventions and best practices for all *.rs files in this workspace. Reference
  when writing new Rust code, reviewing a PR, or deciding between implementation approaches.
  Covers naming, ownership, error handling, API design, testing, and project organisation —
  based on the Rust API Guidelines, RFC 430, and idiomatic community standards.
requires: []
next: []
---

# Rust Conventions

Apply these rules to all Rust code written in this workspace. Prioritise readability, safety,
and maintainability in that order.

---

## Naming (RFC 430)

Follow RFC 430 naming conventions throughout:

| Item                          | Convention             | Example                        |
| ----------------------------- | ---------------------- | ------------------------------ |
| Types, traits, enums          | `UpperCamelCase`       | `TaskGraph`, `PolicyAction`    |
| Functions, methods, variables | `snake_case`           | `check_tool`, `max_calls`      |
| Constants, statics            | `SCREAMING_SNAKE_CASE` | `MAX_NODES`, `DEFAULT_TIMEOUT` |
| Modules                       | `snake_case`           | `mod graph_store`              |
| Lifetimes                     | short lowercase        | `'a`, `'src`                   |
| Type parameters               | short `UpperCamelCase` | `T`, `E`, `Fut`                |

---

## Ownership and Borrowing

- Prefer `&T` over cloning unless ownership transfer is required.
- Use `&mut T` when you need to mutate borrowed data.
- Annotate lifetimes explicitly when the compiler cannot infer them.
- `Rc<T>` for single-threaded reference counting; `Arc<T>` for multi-threaded.
- `RefCell<T>` for interior mutability in single-threaded contexts;
  `Mutex<T>` or `RwLock<T>` for multi-threaded.
- Prefer borrowing and zero-copy operations to avoid unnecessary allocations.
- Use `&str` instead of `String` for function parameters when ownership is not required.

---

## Error Handling

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

---

## Iterators and Collections

- Use iterators instead of index-based loops — they are often faster, safer, and more
  expressive.
- Avoid premature `.collect()` — keep iterator chains lazy until a collection is actually
  needed.
- Prefer `.filter_map()` over `.filter().map()` when the two operations can be combined.
- Use `.fold()` for accumulation rather than a mutable variable outside the loop.

---

## Patterns to Follow

- **Modules and visibility**: Use `mod` + `pub` to encapsulate logic. Keep internal types
  private; expose only what callers need.
- **Enums over flags**: Prefer enum variants over `bool` parameters or integer flags. Type
  safety catches misuse at compile time.
- **Builders for complex construction**: Use the builder pattern when a struct has more than
  3–4 optional fields.
- **Trait abstraction**: Implement traits to abstract services and external dependencies,
  making them replaceable in tests.
- **Async**: Structure async code with `async/await` and `tokio`. Avoid blocking in async
  contexts.
- **Data parallelism**: Use `rayon` for CPU-bound parallel iteration.
- **Binary/library split**: Keep `main.rs` thin — move all logic into `lib.rs` and its
  modules. This enables integration tests and downstream crate reuse.

---

## Patterns to Avoid

- `unwrap()` / `expect()` in non-test, non-prototype code.
- Panics in library code.
- Global mutable state — use dependency injection or thread-safe containers.
- Deeply nested `if`/`match` blocks — extract functions or use combinators.
- Ignoring compiler warnings — treat `-D warnings` as the standard in CI.
- `unsafe` without necessity and thorough documentation.
- Overusing `.clone()` where a borrow would suffice.
- Unnecessary heap allocations in hot paths.

---

## Common Traits

Eagerly derive or implement these where appropriate:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MyType { ... }
```

| Trait                    | When                                         |
| ------------------------ | -------------------------------------------- |
| `Debug`                  | Every public type — required                 |
| `Clone`                  | When callers need owned copies               |
| `PartialEq`, `Eq`        | For value comparison and use in collections  |
| `Hash`                   | When used as a `HashMap`/`HashSet` key       |
| `Default`                | When a zero-value construction is meaningful |
| `Display`                | For user-facing output                       |
| `From` / `Into`          | For idiomatic type conversions               |
| `AsRef`, `AsMut`         | For generic borrowing interfaces             |
| `FromIterator`, `Extend` | For collection types                         |

`Send` and `Sync` are auto-implemented by the compiler. Avoid manual `unsafe impl Send/Sync`
unless you fully understand the invariants.

---

## API Design

- **Newtypes**: Use newtypes to distinguish values that share the same primitive type
  (`struct UserId(u64)` vs `struct OrderId(u64)`).
- **Private fields**: Structs should have private fields by default. Expose state through
  methods to preserve invariants.
- **Sealed traits**: Use the sealed trait pattern to prevent downstream implementations of
  traits not intended for extension.
- **Method receivers**: Functions with a clear receiver should be methods, not free functions.
- **`Deref`/`DerefMut`**: Only smart pointers should implement these. Do not implement `Deref`
  to gain method inheritance.
- **Argument types**: Prefer specific types over generic `bool` parameters. A `bool` argument
  at a call site conveys no intent (`create(true)` vs `create(CreateMode::Overwrite)`).

---

## Async

- Use `tokio` as the async runtime. Do not mix runtimes.
- Never call `.block_on()` inside an async context.
- Use `tokio::spawn` for background tasks; propagate `JoinHandle` errors.
- Prefer `tokio::fs` over `std::fs` in async code.
- Keep `async` boundaries at the edges — pure computation should be synchronous.

---

## Code Style

- Run `rustfmt` (`cargo fmt --all`) before every commit.
- Run `cargo clippy -- -D warnings` and fix all warnings.
- Keep lines at or under 100 characters.
- Place doc comments (`///`) immediately above the item they document.
- Use `//!` for module-level documentation.
- Document error conditions, panic scenarios, and safety considerations.

---

## Testing

- Write unit tests in `#[cfg(test)]` modules in the same file as the code under test.
- Write integration tests in `tests/` with descriptive file names.
- Use `cargo nextest run` (preferred over `cargo test`) for test filtering and parallelism.
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

---

## Project Organisation

- Use semantic versioning in `Cargo.toml`.
- Include `description`, `license`, `repository`, `keywords`, `categories` in every crate.
- Use feature flags for optional functionality.
- Keep `main.rs` and `lib.rs` minimal — move logic to named modules.
- Organise modules as `mod.rs` directory layout (`src/foo/mod.rs`) rather than flat files
  (`src/foo.rs`). This is the preferred convention in this workspace.

---

## Quality Checklist

Before submitting any Rust code:

- [ ] Naming follows RFC 430 throughout
- [ ] All public types derive or implement `Debug`
- [ ] Error handling uses `Result<T, E>` — no bare `unwrap()` in non-test code
- [ ] All public items have `///` rustdoc with at least one sentence
- [ ] Tests cover the happy path and at least one failure/edge case
- [ ] No `unsafe` without a `// SAFETY:` comment explaining the invariant
- [ ] `cargo fmt --all` — no format diff
- [ ] `cargo clippy -- -D warnings` — zero warnings
- [ ] `cargo nextest run` — all green

---

## Related

- `skills/agent-governance/SKILL.md` — Rust implementations of governance patterns
- `skills/systematic-debugging/SKILL.md` — debugging approach for Rust-specific issues
- `skills/verification-before-completion/SKILL.md` — CI gate checklist
- `skills/rust-api-guidelines/SKILL.md` — the full official Rust API Guidelines by C-code, with
  rationale and examples for each rule this document summarizes
- `skills/rust-checklist/SKILL.md` — fast pass/fail checklist audit of a crate's public API
  surface against the official guidelines; use before a release or PR review
