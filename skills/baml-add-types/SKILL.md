---
name: baml-add-types
description: Use when adding new BAML types/functions to crux-baml.
  Symptoms - need to add structured extraction functions, mirror types
  to Rust handler wrappers.
---

# BAML Type Addition

## When to Use

Adding new BAML functions or classes to `crates/crux-baml/baml_src/`.

## Steps

1. Edit the `.baml` file (usually `extract.baml` or `planner.baml`):
   - Add class definitions with fields
   - Add function with client, prompt, and `{{ ctx.output_format }}`
   - Add at least one test case per function

2. Regenerate the BAML client:

   ```bash
   cd crates/crux-baml && mise exec -- baml-cli generate
   ```

3. Verify compilation:

   ```bash
   cargo check -p crux-baml
   ```

4. Add or update the Rust handler wrapper in `crates/crux-baml/src/` and register it from
   `crates/crux-baml/src/lib.rs`:
   - Structs with `#[derive(Debug, Clone, Serialize, Deserialize)]`
   - Use `BTreeMap` not `HashMap` for deterministic serialization
   - Use `Option<T>` for BAML `T?` fields
   - Use `Vec<T>` for BAML `T[]` fields
   - Use `f64` for BAML `float`
   - Use `i64` for BAML `int`

5. Verify the BAML crate compiles and its tests pass:
   ```bash
   cargo check -p crux-baml
   cargo nextest run -p crux-baml
   ```

## Common Failures

| Symptom                       | Fix                                                       |
| ----------------------------- | --------------------------------------------------------- |
| `baml-cli` version mismatch   | Match `generators.baml` version to `Cargo.toml` baml dep  |
| `baml-cli generate` not found | Use `mise exec -- baml-cli generate`, not bare `baml-cli` |
| Generated code won't compile  | Check `baml_client/` is gitignored and regenerate cleanly |
