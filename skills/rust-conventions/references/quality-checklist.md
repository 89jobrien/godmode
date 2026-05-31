# Rust Quality Checklist

Before submitting any Rust code:

- [ ] Naming follows RFC 430 (types=CamelCase, fns=snake_case, consts=SCREAMING_SNAKE)
- [ ] All public types derive or implement `Debug`
- [ ] Error handling uses `Result<T, E>` — no bare `unwrap()` in non-test code
- [ ] All public items have `///` rustdoc with at least one sentence
- [ ] Tests cover the happy path and at least one failure/edge case
- [ ] No `unsafe` without a `// SAFETY:` comment explaining the invariant
- [ ] `cargo fmt --all` — no format diff
- [ ] `cargo clippy -- -D warnings` — zero warnings
- [ ] `cargo nextest run` — all green
- [ ] No dead code, unused imports, or commented-out blocks

## Common Clippy Lints to Watch

| Lint                 | Issue                                        |
| -------------------- | -------------------------------------------- |
| `unnecessary_map_or` | Use `is_ok_and`/`is_some_and` instead        |
| `collapsible_if`     | Merge nested ifs with `&&`                   |
| `needless_return`    | Remove explicit `return` at end of block     |
| `clone_on_copy`      | Don't `.clone()` a Copy type                 |
| `single_match`       | Use `if let` instead of `match` with one arm |

## Error Handling Decision Tree

```
Is this library code?
  YES → Return Result<T, E>, never panic
  NO → Is this a test?
    YES → unwrap() with expect("reason") is acceptable
    NO → Is failure genuinely impossible?
      YES → unwrap() with expect("why impossible")
      NO → Return Result<T, E>
```
