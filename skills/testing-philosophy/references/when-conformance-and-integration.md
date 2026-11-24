# Conformance and integration tests

Read this before writing a test suite for a new `impl Trait` (conformance), or before wiring two crates or modules together and testing the seam (integration).

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
