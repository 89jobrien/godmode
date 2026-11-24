# Clippy lints and error-handling decision tree

Read this before fixing a clippy lint, or before deciding whether a fallible path returns `Result` or panics. The Quality Checklist itself lives in the root `SKILL.md`, which is the single source for that list.

## Common Clippy Lints to Watch

| Lint                 | Issue                                        |
| -------------------- | -------------------------------------------- |
| `unnecessary_map_or` | Use `is_ok_and`/`is_some_and` instead        |
| `collapsible_if`     | Merge nested ifs with `&&`                   |
| `needless_return`    | Remove explicit `return` at end of block     |
| `clone_on_copy`      | Don't `.clone()` a Copy type                 |
| `single_match`       | Use `if let` instead of `match` with one arm |

## Error Handling Decision Tree

```text
Is this library code?
  YES → Return Result<T, E>, never panic
  NO → Is this a test?
    YES → unwrap() with expect("reason") is acceptable
    NO → Is failure genuinely impossible?
      YES → unwrap() with expect("why impossible")
      NO → Return Result<T, E>
```
