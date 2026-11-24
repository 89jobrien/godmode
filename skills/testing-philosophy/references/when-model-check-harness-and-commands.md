# Model checking with Kani: example, comparison, and commands

Read this when writing a Kani proof harness body, running cargo kani locally or in CI, or deciding whether to defer model checking because property tests already cover the invariant.

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
