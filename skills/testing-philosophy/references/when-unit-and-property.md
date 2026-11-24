# Unit and property tests

Read this before writing the first unit test for a new function, or before adding a property test for a function with a non-trivial input space.

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
