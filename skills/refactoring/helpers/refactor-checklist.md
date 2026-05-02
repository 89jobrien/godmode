# Refactoring Checklist

Use this before and after every refactoring pass.

## Before Starting

- [ ] `cargo nextest run --workspace` — baseline is green
- [ ] `cargo clippy --workspace -- -D warnings` — baseline is clean
- [ ] Scope stated: which file(s), which pattern, why
- [ ] No behaviour changes smuggled in

## During

For each structural change:

- [ ] Edit code
- [ ] `cargo nextest run --workspace` — still green
- [ ] If red: revert immediately, diagnose, then retry

## After

- [ ] All tests still pass with identical outcomes
- [ ] No new public API surface unless explicitly approved
- [ ] `cargo fmt --all --check` — no formatting diff
- [ ] `godmode:code-review` run on your own diff
- [ ] One commit per logical change (not one giant refactor commit)

## Scope Creep Check

Before committing, review the diff:

- Am I changing any observable behaviour? → Stop, split into separate commit
- Am I touching files outside the stated scope? → Stop, revert extras
- Am I adding a feature while refactoring? → Stop, do feature separately
