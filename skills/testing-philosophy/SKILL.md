---
name: "godmode:testing-philosophy"
description: >
  The seven-dimension testing model for Rust. Use when designing a test strategy for new
  code, reviewing test coverage, deciding which test type to write next, or onboarding
  to the project's testing conventions. Triggers on "what tests do I need", "how should
  I test this", "test coverage", "fuzz", "kani", "model check", or any question about
  test strategy.
requires: []
next: []
---

# Testing Philosophy

Tests are not a checklist — they are a progression. Each dimension unlocks confidence
at a different stage of the lifecycle. Applying the wrong dimension at the wrong stage
wastes effort. Skipping a dimension leaves a gap that will surface later.

## The Seven Dimensions

```text
Idea → Unit → Property → Fuzz → Model Check → Conformance → Integration → Regression
         ↑         ↑       ↑          ↑              ↑              ↑            ↑
      always   non-trivial |       arithmetic      new impl        wiring       bug fixed
               input space unsafe/parser           overflow/       complete
                           boundary                invariant
```

| Dimension   | Lifecycle stage                        | Question it answers                                                  |
| ----------- | -------------------------------------- | -------------------------------------------------------------------- |
| Unit        | Design — function exists               | Does this function do what I think it does?                          |
| Property    | Design — input space understood        | Does this invariant hold for all valid inputs?                       |
| Fuzz        | Design — unsafe/parser/protocol code   | Does this code survive malformed or adversarial input?               |
| Model Check | Design — correctness proof needed      | Can this function violate safety invariants for any reachable input? |
| Conformance | Design — trait contract defined        | Does this impl satisfy the contract the trait promises?              |
| Integration | Build — components wired together      | Do these parts work correctly when connected?                        |
| Regression  | Maintenance — bug reproduced and fixed | Will this specific failure mode ever recur?                          |

## When to Apply Each

Read the file named in a row before applying the dimension in its right column. These files
are not loaded automatically.

| File                                             | Read before                                                                                  |
| ------------------------------------------------ | -------------------------------------------------------------------------------------------- |
| `references/when-unit-and-property.md`           | writing the first unit test for a function, or a property test for a non-trivial input space |
| `references/when-fuzz.md`                        | adding a fuzz target for parsing, untrusted input, or `unsafe` code                          |
| `references/when-model-check.md`                 | writing a Kani proof harness, or deciding whether a function needs a proof                   |
| `references/when-conformance-and-integration.md` | writing a conformance suite for an `impl Trait`, or testing a seam between components        |
| `references/when-regression.md`                  | fixing a bug, before the fix lands                                                           |

---

## Rules

- Do not skip Unit to write Integration first — integration tests on unverified units
  hide which component is wrong
- Do not write Property tests before understanding the invariant — `proptest` finds
  counterexamples; you must know what to assert
- Fuzz targets are required for any function that parses external bytes or contains `unsafe`
- Fuzz corpus files in `fuzz/corpus/` are permanent — never delete them
- Model check proofs are required for any function that performs arithmetic on external inputs
- `kani::assume()` bounds must reflect real domain constraints — over-constraining hides bugs
- Conformance tests belong to the trait, not the impl — one suite, multiple impls
- Regression tests are permanent — never delete a regression file or test
- `unwrap()` in tests must have `expect("why this can't fail")` — no silent panics

## Additional Resources

- **`references/dimension-examples.md`** — concrete Rust examples for each dimension (unit,
  property, fuzz, model check, conformance, integration, regression)
- **`helpers/test-scaffold.nu <crate> <dimension>`** — generate test module stubs
  (run via `nu helpers/test-scaffold.nu <crate> <dimension>`)
- To check whether a `.rs` file has associated tests, use Grep tool to search for
  `#\[test\]` or `#\[cfg\(test\)\]` in the file's directory.
