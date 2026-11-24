# Detection categories

Read this before classifying a recurring failure, or when deciding which of the five categories a new pattern belongs to.

## 1. Clippy Lint Recurrence

Repeated clippy warnings with the same lint ID appearing across multiple sessions or
commits. Indicates a pattern the codebase is prone to that needs either:

- Systemic suppression (if false positive or accepted)
- Refactoring (if genuine pattern to eliminate)
- Compiler upgrade (if lint behavior changed)

Example:

```text
Pattern: clippy::too_many_arguments
Occurrences: 5 (sessions 2026-05-10, 2026-05-12, 2026-05-15)
Affected: crates/godmode-core/src/session.rs
Prevention: Apply deny rule or refactor functions to use builder pattern
```
