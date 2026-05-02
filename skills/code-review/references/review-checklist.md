# Code Review Checklist

## Correctness

- [ ] Logic matches the stated requirement
- [ ] Edge cases handled: empty input, null/None, zero, overflow
- [ ] Error paths return meaningful errors, not panics
- [ ] No silent data loss (truncation, lossy casts, `.unwrap()`)

## Safety & Security

- [ ] No SQL/command injection via string interpolation
- [ ] Secrets not logged, serialised, or in error messages
- [ ] File paths validated before use
- [ ] No `unsafe` block without a `// SAFETY:` comment explaining why
- [ ] User input not passed unescaped to shell commands

## Architecture

- [ ] Change is in the right layer (I/O not mixed into pure logic)
- [ ] No new circular dependencies between crates
- [ ] Public API surface is intentional — not accidental `pub` leakage
- [ ] Hexagonal boundary respected: ports define contracts, adapters implement them
- [ ] New external dependency behind a trait (port)

## Tests

- [ ] Every new public function has at least one test
- [ ] Happy path covered
- [ ] At least one error/edge case covered
- [ ] Tests use real code (in-memory fakes, not mock frameworks)
- [ ] No test-only methods on production types
- [ ] Integration tests use `tests/` dir, not `#[cfg(test)]` in lib

## Style

- [ ] Names clear and consistent with surrounding code
- [ ] No dead code, unused imports, commented-out blocks
- [ ] Doc comments on public items where non-obvious
- [ ] Line width ≤ 100 columns
- [ ] `cargo clippy -- -D warnings` clean

## Severity Guide

| Finding                               | Severity   |
| ------------------------------------- | ---------- |
| Logic error, data loss, panic risk    | Blocking   |
| Missing test for public function      | Blocking   |
| Unclear name, minor duplication       | Suggestion |
| Formatting, minor style inconsistency | Nitpick    |
| False positive on test fixture        | Allowlist  |
