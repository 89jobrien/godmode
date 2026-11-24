# Common traits and API design

Read this before adding derives or trait impls to a public type, or when designing a public API surface: newtypes, visibility, sealed traits, receivers, and argument types.

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
