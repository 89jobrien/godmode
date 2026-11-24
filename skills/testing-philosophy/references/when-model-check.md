# Model checking with Kani

Read this before writing a Kani proof harness, or before deciding whether a function needs a proof rather than a sampled test.

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
