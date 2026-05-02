---
name: godmode:code-review
description: >
  Structured code review before merge. Use when implementation is complete and you need
  a systematic quality pass — correctness, safety, architecture, tests, and style.
---

# Code Review

## When to Use

- Before creating a PR or merging a branch
- After a feature is implemented and tests pass
- When asked to "review", "audit", or "check" code

## Severity Levels

| Level      | Action                          |
| ---------- | ------------------------------- |
| Blocking   | Must fix before merge           |
| Suggestion | Should fix; explain if skipping |
| Nitpick    | Optional; fix in one pass       |

**Apply all severity levels in one pass.** Do not commit after fixing only blocking
issues and leave suggestions for a follow-up — that creates noisy fix histories.

## Review Checklist

### Correctness

- [ ] Logic matches the stated requirement
- [ ] Edge cases handled: empty input, null/None, zero, overflow
- [ ] Error paths return meaningful errors, not panics
- [ ] No silent data loss (truncation, lossy casts)

### Safety & Security

- [ ] No SQL/command injection via string interpolation
- [ ] Secrets not logged or serialised
- [ ] File paths validated before use
- [ ] No `unsafe` without justification

### Architecture

- [ ] Change is in the right layer (not mixing I/O into pure logic)
- [ ] No new circular dependencies
- [ ] Public API surface is intentional — not accidental leakage
- [ ] Hexagonal boundary respected: ports define contracts, adapters implement them

### Tests

- [ ] Every new public function has at least one test
- [ ] Happy path covered
- [ ] At least one error/edge case covered
- [ ] Tests use real code (mocks only where unavoidable)
- [ ] No test-only methods on production types

### Style

- [ ] Names are clear and consistent with surrounding code
- [ ] No dead code, unused imports, commented-out blocks
- [ ] Doc comments on public items where non-obvious
- [ ] Line width ≤ 100 columns

## Process

1. Read the diff or specified files completely before commenting
2. Group findings by severity
3. Report as structured list:

```
## Code Review — <file or feature>

### Blocking
- [file:line] <issue> — <why it matters>

### Suggestions
- [file:line] <issue> — <recommendation>

### Nitpicks
- [file:line] <issue>
```

4. Fix all issues in one pass before marking done
5. Re-run `cargo clippy --workspace -- -D warnings` and `cargo test --workspace` after fixes
6. Use `godmode:verification-before-completion` before claiming done

## False Positives

When a reviewer (sentinel, clippy, obfsck) flags test data, string literals, or fixture
content:

- Add a per-site `#[allow(...)]` or allowlist entry immediately
- Do not change test content to work around the flag
- Document why the allowlist entry was added
