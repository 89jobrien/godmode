# Refactoring Checklist

Use this before and after every refactoring pass.

## Before Starting

- [ ] `taskit test run` — baseline is green
- [ ] `taskit check lint` — baseline is clean
- [ ] Scope stated: which file(s), which pattern, why
- [ ] No behaviour changes smuggled in

## During

For each structural change:

- [ ] Edit code
- [ ] `taskit test run` — still green
- [ ] If red: revert immediately, diagnose, then retry

## After

- [ ] All tests still pass with identical outcomes
- [ ] No new public API surface unless explicitly approved
- [ ] `taskit check fmt --check` — no formatting diff
- [ ] `godmode:code-review` run on your own diff
- [ ] One commit per logical change (not one giant refactor commit)

## Scope Creep Check

Before committing, review the diff:

- Am I changing any observable behaviour? → Stop, split into separate commit
- Am I touching files outside the stated scope? → Stop, revert extras
- Am I adding a feature while refactoring? → Stop, do feature separately
