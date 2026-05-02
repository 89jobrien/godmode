# Design Principles Reference

## Hexagonal Architecture

External dependencies belong behind a trait (port) in the domain layer. Implementations
live in adapters (`infra/` or equivalent). Business logic is generic over trait bounds.

```
domain/
  ports/          ← trait definitions (interfaces)
  services/       ← business logic, generic over ports
infra/
  adapters/       ← concrete implementations (HTTP, DB, FS)
```

Rule: domain files have zero imports from infrastructure crates.

## YAGNI

Remove anything not needed for the current requirement. Questions to ask:

- "Would we ship without this?"
- "Is there a test that requires this?"
- "Is there a user story for this?"

If all three are no — delete it.

## Rust Idioms

| Pattern          | Prefer                         | Avoid                       |
| ---------------- | ------------------------------ | --------------------------- |
| Error handling   | `Result<T, E>`, `?` operator   | `unwrap()`, `expect()`      |
| Trait objects    | `impl Trait` (static dispatch) | `Box<dyn Trait>` where able |
| Struct defaults  | `#[derive(Default)]`           | Manual `Default` impls      |
| Iteration        | Iterator adapters              | Manual index loops          |
| String ownership | `&str` in function params      | `String` params             |

## Rust-Specific Design Questions

Before proposing any design, answer these:

1. **Crate ownership**: Which crate owns this? Existing or new?
2. **Dependencies**: Any new external crates? Can they be feature-flagged?
3. **Testing strategy**: In-memory fakes or integration tests?
4. **API surface**: Public or private? Semver implications?
5. **Trait boundaries**: What traits does this need to implement or accept?
