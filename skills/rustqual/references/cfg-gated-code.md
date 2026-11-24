# cfg-gated code

Read this before deciding how to handle a rustqual finding that involves `#[cfg(kani)]` modules or cfg-gated test utilities such as `#[cfg(any(test, feature = "testutil"))]`.

`#[cfg(kani)]` modules and `#[cfg(any(test, feature = "testutil"))]` code
are invisible to rustqual's dead-code analysis. Two approaches:

1. **Extract to dedicated files** and add to `exclude_files` in config:

   ```toml
   exclude_files = ["src/kani_proofs.rs", "src/testutil.rs"]
   ```

   Use `#[path = "kani_proofs.rs"]` to keep the module relationship.

2. **Disable dead code detection** if ALL dead-code findings are cfg-gated:

   ```toml
   [duplicates]
   detect_dead_code = false
   ```

Option 1 is preferred — it keeps detection active for real dead code while
excluding known false-positive files.
