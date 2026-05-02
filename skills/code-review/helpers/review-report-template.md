# Code Review — <file or feature>

**Reviewer**: <name>
**Date**: YYYY-MM-DD
**Commit / PR**: <sha or PR link>

## Summary

One sentence: overall quality and readiness to merge.

---

## Blocking

Issues that must be fixed before merge.

- [file:line] <issue> — <why it matters>

## Suggestions

Should fix; explain if skipping.

- [file:line] <issue> — <recommendation>

## Nitpicks

Optional; fix in one pass if touching the file anyway.

- [file:line] <issue>

---

## Checklist

### Correctness

- [ ] Logic matches stated requirement
- [ ] Edge cases handled (empty input, None, zero, overflow)
- [ ] Error paths return meaningful errors, not panics
- [ ] No silent data loss

### Safety & Security

- [ ] No injection via string interpolation
- [ ] Secrets not logged or serialised
- [ ] File paths validated before use
- [ ] No unjustified `unsafe`

### Architecture

- [ ] Change is in the right layer
- [ ] No new circular dependencies
- [ ] Public API surface is intentional
- [ ] Hexagonal boundary respected

### Tests

- [ ] Every new public function has at least one test
- [ ] Happy path covered
- [ ] At least one error/edge case covered
- [ ] No test-only methods on production types

### Style

- [ ] Names clear and consistent
- [ ] No dead code or unused imports
- [ ] Doc comments on non-obvious public items
- [ ] Line width <= 100 columns

---

## Verdict

- [ ] Approve — ready to merge
- [ ] Approve with minor fixes (listed above as suggestions/nitpicks)
- [ ] Request changes — blocking issues must be resolved first
