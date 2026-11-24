# Naming and ownership

Read this before naming a new type, function, constant, module, or lifetime, or when deciding between borrowing, cloning, and shared ownership.

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
