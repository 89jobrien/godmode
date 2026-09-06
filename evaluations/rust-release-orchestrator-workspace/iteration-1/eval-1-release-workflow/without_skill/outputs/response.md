# Releasing the kan workspace to crates.io

## Overview

The workspace has three crates with a strict dependency chain:

```
kan-core  →  kan-adapters  →  kan-cli
```

crates.io requires each crate to be published independently, in dependency order. You cannot publish `kan-adapters` until `kan-core` is live, and you cannot publish `kan-cli` until both `kan-core` and `kan-adapters` are live. The `path` dependencies in Cargo.toml are automatically resolved to the published versions on crates.io after each publish.

All three crates currently share `version = "0.1.0"` via `workspace.package`.

---

## Pre-flight checklist (run these first)

```sh
# 1. Verify you are logged in to crates.io
cargo login

# 2. Clean build and test pass
cargo build --workspace --release
cargo nextest run

# 3. Clippy clean (CI gate)
cargo clippy --all-targets -- -D warnings

# 4. Fmt check
cargo fmt --all --check

# 5. License / advisory audit
cargo deny check

# 6. Confirm git state is clean and on main
git status
git log --oneline -3
```

---

## Step 1: Dry run (no network writes)

Run each crate in order with `--dry-run`. This validates metadata, checks for missing fields, and simulates the upload without publishing anything.

```sh
# 1a. Dry-run kan-core
cargo publish -p kan-core --dry-run

# 1b. Dry-run kan-adapters
cargo publish -p kan-adapters --dry-run

# 1c. Dry-run kan-cli
cargo publish -p kan-cli --dry-run
```

### What to watch for in dry-run output

- **Missing metadata** — `description`, `repository`, `homepage`, `documentation`, `keywords`, `categories` are not required but crates.io will warn on missing `description` and `repository`. You currently have none of these in `[workspace.package]`. Add them before the real publish.
- **Path deps warning** — `cargo publish` replaces `path = "../kan-core"` with the version from crates.io automatically. Dry-run will confirm this substitution happens cleanly.
- **Bundled feature on rusqlite** — `rusqlite = { features = ["bundled"] }` inflates the package size significantly and links libsqlite3 statically. That is fine but will produce a large `.crate` file; expected.
- **`serde_yaml` in kan-cli** — `serde_yaml` (0.9) is unmaintained upstream. Not a blocker for publish, but `cargo deny check` may flag it depending on your `deny.toml` configuration.

---

## Step 2: Add missing Cargo.toml metadata (before real publish)

`[workspace.package]` in `Cargo.toml` should include at minimum:

```toml
[workspace.package]
version = "0.1.0"
edition = "2024"
license = "MIT OR Apache-2.0"
description = "Kanban-style project management TUI and CLI"
repository = "github:89jobrien/kan"
```

Each crate inherits these via `.workspace = true`. No per-crate edits needed unless a crate needs a distinct description (e.g., `kan-cli` might want a more specific one).

---

## Step 3: Real publish (in order)

After the dry-run passes and metadata is in place:

```sh
# 3a. Publish kan-core — must be first, no local deps
cargo publish -p kan-core

# Wait ~30 seconds for crates.io index propagation before the next step.
# kan-adapters depends on the published kan-core, so the registry must
# have it indexed before cargo can resolve it.
sleep 30

# 3b. Publish kan-adapters
cargo publish -p kan-adapters

# Wait again for index propagation
sleep 30

# 3c. Publish kan-cli
cargo publish -p kan-cli
```

### Why the sleeps?

After `cargo publish` succeeds, crates.io updates its sparse index within ~10–30 seconds. If you publish `kan-adapters` immediately after `kan-core`, cargo may fail to resolve `kan-core = "0.1.0"` from the registry because the index hasn't caught up yet. The sleep is a simple guard; you can also watch https://crates.io/crates/kan-core and proceed once it shows the new version.

---

## Step 4: Verify

```sh
# Confirm all three are live on crates.io
cargo search kan-core
cargo search kan-adapters
cargo search kan-cli
```

Or check directly:

- https://crates.io/crates/kan-core
- https://crates.io/crates/kan-adapters
- https://crates.io/crates/kan-cli

---

## Step 5: Tag the release

```sh
git tag -s v0.1.0 -m "Release v0.1.0"
git push origin v0.1.0
```

---

## Summary of command sequence

```sh
# Pre-flight
cargo build --workspace --release
cargo nextest run
cargo clippy --all-targets -- -D warnings
cargo fmt --all --check
cargo deny check

# Dry run (in dep order)
cargo publish -p kan-core --dry-run
cargo publish -p kan-adapters --dry-run
cargo publish -p kan-cli --dry-run

# Real publish (in dep order, with index propagation waits)
cargo publish -p kan-core
sleep 30
cargo publish -p kan-adapters
sleep 30
cargo publish -p kan-cli

# Tag
git tag -s v0.1.0 -m "Release v0.1.0"
git push origin v0.1.0
```

---

## Known gaps to address before publishing

1. **Missing metadata**: `description` and `repository` fields are absent from `[workspace.package]`. Add them — crates.io will reject without `description`.
2. **`serde_yaml` 0.9** in `kan-cli` is unmaintained. Consider switching to `serde_json` or `toml` if `cargo deny check` blocks on it.
3. **No `README` field**: Adding `readme = "README.md"` to `[workspace.package]` (and creating a `README.md` in the workspace root) gives crates.io a rendered page for each crate.
4. **`kan-cli` publishes two binaries** (`kan` and `kan-cli`). Both will be installed via `cargo install kan-cli`. That is correct — just confirm the binary names are what you want.
