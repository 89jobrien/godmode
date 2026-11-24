# Patterns to follow and avoid

Read this before structuring a new module or type, or when reviewing code for design smells such as globals, deep nesting, or needless clones.

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
