# Finding codes: DRY and SRP

Read this before fixing a DEAD_CODE, BOILERPLATE, SRP_STRUCT, or SRP_MODULE finding, or before changing detect_dead_code.

### DRY Findings

- **DEAD_CODE** (testonly): A function is only called from test code within
  the same crate. This is a frequent false positive for pub API methods used
  by other crates in the workspace.

  **Important**: DEAD_CODE cannot be suppressed with `// qual:allow(dry)`.
  rustqual will flag the allow comment as an ORPHAN_SUPPRESSION. Instead,
  disable dead code detection in `rustqual.toml` if all findings are
  cross-crate false positives:

  ```toml
  [duplicates]
  detect_dead_code = false
  ```

- **BOILERPLATE** (BP-002): Manual trait impl that could use a derive macro.
  Common for `Display`, `Error`, `From` impls.

  **Fix pattern**: Use `thiserror::Error` derive instead of manual
  `impl Display + impl Error`.

### SRP Findings

- **SRP_STRUCT** (LCOM4=N): Struct has low cohesion — its methods form N
  independent clusters. Methods in different clusters don't share fields.

  **Fix pattern**: Split the struct into separate types, or extract a group
  of methods into a standalone function that takes the struct as a parameter.

- **SRP_MODULE** (N lines): File exceeds the configured length ceiling.

  **Fix pattern**: Split into submodules. For test files, use `mod init {}`,
  `mod graph {}`, etc.
