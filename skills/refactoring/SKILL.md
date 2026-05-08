---
name: "godmode:refactoring"
description: >
  Guided refactoring with test discipline. Use when restructuring code without changing
  observable behaviour — extract, rename, reorganise, decouple.
---

# Refactoring

## Core Rule

```
REFACTOR ONLY WHEN ALL TESTS PASS
Never change behaviour and structure at the same time.
```

If tests are red before you start, fix them first. Refactoring on a red baseline hides
regressions.

## When to Use

- Extracting a helper that is duplicated 3+ times
- Renaming for clarity after the design has stabilised
- Decoupling I/O from pure logic (hexagonal prep)
- Removing dead code after a feature is complete

## When NOT to Use

- When you're tempted to refactor and add a feature simultaneously — split the work
- When tests don't exist yet — write tests first (`godmode:task-driven-development`)
- When the "refactor" would change public API behaviour — that's a feature change

## Process

### 1. Confirm green baseline

> Run via `nu skills/_lib/quality-gate.nu run-quality-gate` or use the commands below:

```bash
cargo nextest run --workspace
cargo clippy --workspace -- -D warnings
```

Both must pass before touching anything.

### 2. State scope

Name exactly what you're changing and why:

- Which file(s)
- What pattern (extract, rename, move, inline, decouple)
- Why (duplication, clarity, coupling reduction)

Do not expand scope without surfacing to user.

### 3. Make one structural change at a time

Each change:

1. Edit code
2. Run `cargo nextest run --workspace` — must stay green
3. If red, revert and diagnose before proceeding

### 4. Verify behaviour unchanged

- Tests pass with identical outcomes
- No new public API surface unless explicitly approved
- No behaviour changes smuggled into the refactor

### 5. Run full gate

> Run via `nu skills/_lib/quality-gate.nu run-quality-gate` or use the commands below:

```bash
cargo fmt --all --check
cargo clippy --workspace -- -D warnings
cargo nextest run --workspace
```

## Common Refactoring Patterns (Rust)

| Pattern          | Signal                               | Approach                              |
| ---------------- | ------------------------------------ | ------------------------------------- |
| Extract function | Block of code used 2+ times          | Move to `fn`, pass args               |
| Extract module   | File > ~300 lines, mixed concerns    | Split by responsibility               |
| Extract trait    | Two types share identical method set | Define trait, impl on each            |
| Inline variable  | Variable used once, name adds noise  | Inline the expression                 |
| Decouple I/O     | Pure logic calls `std::fs`           | Pass `&Path` or `Read` trait bound    |
| Rename           | Name misleads or contradicts usage   | `sed`/IDE rename, check all callsites |

## Scope Rules

- Do not rename public items without checking all downstream callsites
- Do not move items across crate boundaries without updating `Cargo.toml` deps
- Do not combine rename + extract in one step — rename first, verify green, then extract

## After Refactoring

Run `godmode:code-review` on your own diff before committing.

## Additional Resources

- **`references/refactoring-patterns.md`** — extract function/module/trait, decouple I/O, rename patterns with code examples
- **`helpers/refactor-checklist.md`** — before/during/after checklist and scope creep check
