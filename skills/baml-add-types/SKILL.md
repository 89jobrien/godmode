---
name: baml-add-types
description: Use when adding new BAML types/functions to cruxx-agentic.
  Symptoms - need to add structured extraction functions, mirror types
  to cruxx-types.
---

# BAML Type Addition

## When to Use

Adding new BAML functions or classes to `cruxx-agentic/baml_src/`.

## Steps

1. Edit the `.baml` file (usually `extract.baml` or `planner.baml`):
   - Add class definitions with fields
   - Add function with client, prompt, and `{{ ctx.output_format }}`
   - Add at least one test case per function

2. Regenerate the BAML client:

   ```bash
   cd crates/cruxx-agentic && mise exec -- baml-cli generate
   ```

3. Verify compilation:

   ```bash
   cargo check -p cruxx-agentic
   ```

4. Mirror shared output types to `cruxx-types::extraction`
   (`crates/cruxx-types/src/extraction.rs`):
   - Structs with `#[derive(Debug, Clone, Serialize, Deserialize)]`
   - Use `BTreeMap` not `HashMap` for deterministic serialization
   - Use `Option<T>` for BAML `T?` fields
   - Use `Vec<T>` for BAML `T[]` fields
   - Use `f64` for BAML `float`
   - Use `i64` for BAML `int`

5. Verify cruxx-types compiles:
   ```bash
   cargo check -p cruxx-types
   ```

## Common Failures

| Symptom                       | Fix                                                       |
| ----------------------------- | --------------------------------------------------------- |
| `baml-cli` version mismatch   | Match `generators.baml` version to `Cargo.toml` baml dep  |
| `baml-cli generate` not found | Use `mise exec -- baml-cli generate`, not bare `baml-cli` |
| Generated code won't compile  | Check `baml_client/` is gitignored and regenerate cleanly |
