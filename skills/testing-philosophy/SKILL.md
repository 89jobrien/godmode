---
name: godmode:testing-philosophy
description: >
  The five-dimension testing model for Rust. Use when designing a test strategy for new
  code, reviewing test coverage, deciding which test type to write next, or onboarding
  to the project's testing conventions. Triggers on "what tests do I need", "how should
  I test this", "test coverage", or any question about test strategy.
---

# Testing Philosophy

Tests are not a checklist — they are a progression. Each dimension unlocks confidence
at a different stage of the lifecycle. Applying the wrong dimension at the wrong stage
wastes effort. Skipping a dimension leaves a gap that will surface later.

## The Five Dimensions

```
Idea → Unit → Property → Conformance → Integration → Regression
         ↑         ↑            ↑              ↑            ↑
      always   non-trivial   new impl      wiring        bug fixed
               input space   Trait         complete
```

| Dimension   | Lifecycle stage                        | Question it answers                                     |
| ----------- | -------------------------------------- | ------------------------------------------------------- |
| Unit        | Design — function exists               | Does this function do what I think it does?             |
| Property    | Design — input space understood        | Does this invariant hold for all valid inputs?          |
| Conformance | Design — trait contract defined        | Does this impl satisfy the contract the trait promises? |
| Integration | Build — components wired together      | Do these parts work correctly when connected?           |
| Regression  | Maintenance — bug reproduced and fixed | Will this specific failure mode ever recur?             |

## When to Apply Each

### Unit — always, first

Write unit tests the moment a function exists. They are the foundation everything else
builds on. A function without a unit test has no verified behaviour.

- Scope: one function, pure logic
- Fakes over mocks — pass a `Vec` not a `MockRepository`
- Live in `#[cfg(test)]` in the same file
- Name: `fn <thing>_<scenario>_<expected>()`

### Property — when the input space is non-trivial

If a function accepts strings, integers, collections, or any type with many possible
values, a unit test only proves it works for the inputs you thought of. Property tests
prove invariants hold for inputs you didn't think of.

- Use `proptest` crate; strategies via `prop_compose!` for domain types
- Test invariants, not specific values: "output is always sorted", "round-trip is lossless"
- Commit `proptest-regressions/` — these are found counterexamples, never delete them
- Good candidates: parsers, graph operations, serialisation, arithmetic, sorting

### Conformance — for every new `impl Trait`

A trait defines a contract. An impl that compiles is not necessarily correct — it may
violate the semantic invariants the trait promises. Conformance tests verify the impl,
not just the function.

Pattern:

```rust
// In tests/conformance_<trait>.rs or a shared test module
fn assert_port_contract<T: MyPort>(impl_under_test: T) {
    // assert every invariant the trait doc promises
}

#[test]
fn my_adapter_satisfies_port_contract() {
    assert_port_contract(MyAdapter::new());
}
```

- One conformance test suite per trait, reusable across all impls
- Tests the _contract_, not the implementation details
- Required for every port (hexagonal architecture adapter)

### Integration — when wiring is complete

Integration tests verify that components work correctly when connected. They are slower
and more expensive than unit tests — write them after the components are individually
verified, not before.

- Live in `tests/` dir (separate compilation unit)
- Use real I/O boundaries where the test is specifically about that boundary
- One integration test per cross-crate or cross-module seam
- Do not duplicate unit test coverage — test the wiring, not the logic

### Regression — after every bug fix

Every bug that reaches production or a failing test represents a gap in the test suite.
Close that gap permanently.

1. Reproduce the bug with a minimal failing test (unit or property)
2. Fix the bug
3. Verify the test now passes
4. If it was a property test failure, commit the regression file

Never fix a bug without a regression test. "It's fixed" without a test means it will
recur.

## Rules

- Do not skip Unit to write Integration first — integration tests on unverified units
  hide which component is wrong
- Do not write Property tests before understanding the invariant — `proptest` finds
  counterexamples; you must know what to assert
- Conformance tests belong to the trait, not the impl — one suite, multiple impls
- Regression tests are permanent — never delete a regression file or test
- `unwrap()` in tests must have `expect("why this can't fail")` — no silent panics

## Additional Resources

- **`references/dimension-examples.md`** — concrete Rust examples for each dimension
- **`helpers/test-scaffold.nu`** — generate test module stubs for a given crate and dimension
