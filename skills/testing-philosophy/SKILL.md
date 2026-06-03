---
name: "godmode:testing-philosophy"
description: >
  The seven-dimension testing model for Rust. Use when designing a test strategy for new
  code, reviewing test coverage, deciding which test type to write next, or onboarding
  to the project's testing conventions. Triggers on "what tests do I need", "how should
  I test this", "test coverage", "fuzz", "kani", "model check", or any question about
  test strategy.
---

# Testing Philosophy

Tests are not a checklist — they are a progression. Each dimension unlocks confidence
at a different stage of the lifecycle. Applying the wrong dimension at the wrong stage
wastes effort. Skipping a dimension leaves a gap that will surface later.

## The Seven Dimensions

```
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

### Fuzz — for unsafe, parsers, and protocol boundaries

If a function touches raw bytes, parses external input, handles untrusted data, or
contains `unsafe`, fuzz it. Property tests generate structured inputs; fuzz tests
generate arbitrary byte sequences. They find panics, integer overflows, and logic
errors that structured generation misses.

- Use `cargo-fuzz` (libFuzzer); targets live in `fuzz/fuzz_targets/`
- The `fuzz/` crate must be excluded from the workspace (`workspace.exclude = ["fuzz"]`)
- Seed corpus in `fuzz/corpus/<target>/` — commit representative inputs
- Requires nightly: `cargo +nightly fuzz run <target> -- -max_total_time=60`
- Good candidates: markdown/YAML/TOML parsers, template substitution, any function
  that deserialises external strings, any function that calls `unsafe`
- A fuzz target that finds no crash in 60s on first run is table stakes, not done —
  add it to CI with a short time budget (`-max_total_time=30`)
- Assert lightweight invariants inside fuzz targets, not just crash-freedom — e.g.
  verify summary counts equal task count after deserialising arbitrary YAML

```rust
// fuzz/fuzz_targets/fuzz_plan_parse.rs
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // plan::parse must never panic on arbitrary markdown input.
        let _ = godmode_core::plan::parse(s);
    }
});
```

### Model Check — for proving absence of bugs

When fuzz testing reduces the input space statistically, model checking exhausts it
symbolically within bounds. Use Kani when you need to prove that a function cannot violate
a safety invariant for any reachable input — not just inputs you sampled.

Apply when:

- A function performs arithmetic on external inputs that could overflow
- Array or slice indexing is conditional on runtime values
- The code contains `unsafe` blocks with invariants that must hold
- A state machine has reachability invariants that must be proven, not just tested
- You need to guarantee "no possible input causes X" rather than "X hasn't happened yet"

Setup:

- Add `kani` crate as a dev-dependency or use the cargo-kani subcommand
- Proof harnesses live in `#[cfg(kani)]` modules alongside the code they verify
- Use `kani::any()` for symbolic inputs — like proptest strategies but exhaustive within bounds
- Bound loops with `#[kani::unwind(N)]`; start low and increase until the proof is meaningful

```rust
#[cfg(kani)]
mod kani_proofs {
    use super::*;

    #[kani::proof]
    #[kani::unwind(5)]
    fn check_no_overflow_in_score() {
        let a: u32 = kani::any();
        let b: u32 = kani::any();
        kani::assume(a <= 1000);
        kani::assume(b <= 1000);
        let result = compute_score(a, b);
        assert!(result <= 2000);
    }
}
```

Good candidates: score calculations, index arithmetic, state transitions, protocol framing
sizes, any function with `checked_add`/`saturating_mul` or that should use them.

Difference from fuzz: fuzz finds bugs by sampling arbitrary inputs; Kani proves absence by
exhaustive symbolic execution within the specified bounds. They are complementary — fuzz first
to find crashes, then model-check to prove the fixed invariant holds for all inputs.

Run locally:

```bash
cargo kani --harness check_no_overflow_in_score
```

CI integration:

```bash
cargo kani --output-format terse
```

**Note:** Kani has significant setup cost. If property tests already cover the same
invariants with `proptest`, defer model checking until a correctness bug surfaces that
property tests missed.

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
- **`godmode scaffold <crate> <dimension>`** — generate test module stubs (Rust impl in
  `godmode-core/src/scaffold.rs`; `helpers/test-scaffold.nu` is a thin shim)
- **`godmode test-check <path>`** — check whether a `.rs` file has associated tests (Rust
  impl in `godmode-core/src/test_check.rs`; `hook.nu` is a thin shim)
