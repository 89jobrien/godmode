# Project organisation

Read this before creating a crate, editing `Cargo.toml`, adding a feature flag, or laying out modules.

- Use semantic versioning in `Cargo.toml`.
- Include `description`, `license`, `repository`, `keywords`, `categories` in every crate.
- Use feature flags for optional functionality.
- Keep `main.rs` and `lib.rs` minimal — move logic to named modules.
- Organise modules as `mod.rs` directory layout (`src/foo/mod.rs`) rather than flat files
  (`src/foo.rs`). This is the preferred convention in this workspace.
