---
name: "godmode:rust-conventions"
description: >
  Rust coding conventions and best practices for all *.rs files in this workspace. Reference
  when writing new Rust code, reviewing a PR, or deciding between implementation approaches.
  Covers naming, ownership, error handling, API design, testing, and project organisation —
  based on the Rust API Guidelines, RFC 430, and idiomatic community standards.
requires: []
next: []
---

# Rust Conventions

Apply these rules to all Rust code written in this workspace. Prioritise readability, safety,
and maintainability in that order.

## Reference files

Read the file named in a row before doing the task in its right column. These files are not
loaded automatically.

| File                                           | Read before                                                                                       |
| ---------------------------------------------- | ------------------------------------------------------------------------------------------------- |
| `references/naming-and-ownership.md`           | naming a new item, or choosing between borrowing, cloning, and shared ownership                   |
| `references/error-handling.md`                 | writing a fallible function, defining an error type, or deciding whether code may panic           |
| `references/iterators-and-collections.md`      | writing a loop, an iterator chain, or an accumulation                                             |
| `references/patterns-to-follow-and-avoid.md`   | structuring a new module or type, or reviewing for design smells                                  |
| `references/traits-and-api-design.md`          | adding derives or trait impls to a public type, or designing a public API                         |
| `references/async.md`                          | writing or changing async code or background tasks                                                |
| `references/code-style-and-testing.md`         | before committing, writing doc comments, or adding tests                                          |
| `references/project-organisation.md`           | creating a crate, editing `Cargo.toml`, or laying out modules                                     |
| `references/common-patterns.md`                | choosing a construction or structuring pattern such as the builder pattern                        |
| `references/clippy-and-error-decision-tree.md` | a clippy lint fires, or you are deciding between returning `Result` and calling `unwrap`/`expect` |

---

## Quality Checklist

Before submitting any Rust code:

- [ ] Naming follows RFC 430 throughout
- [ ] All public types derive or implement `Debug`
- [ ] Error handling uses `Result<T, E>` — no bare `unwrap()` in non-test code
- [ ] All public items have `///` rustdoc with at least one sentence
- [ ] Tests cover the happy path and at least one failure/edge case
- [ ] No `unsafe` without a `// SAFETY:` comment explaining the invariant
- [ ] `taskit check fmt` — no format diff
- [ ] `taskit check lint` — zero warnings
- [ ] `taskit test run` — all green
- [ ] No dead code, unused imports, or commented-out blocks

---

## Related

- `skills/agent-governance/SKILL.md` — Rust implementations of governance patterns
- `skills/systematic-debugging/SKILL.md` — debugging approach for Rust-specific issues
- `skills/verification-before-completion/SKILL.md` — CI gate checklist
