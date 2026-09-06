# Resuming an Interrupted Workspace Release (kan)

## Situation

`kan-core` was published successfully. `kan-adapters` and `kan-cli` were not.

## Why you cannot re-publish kan-core

`cargo publish` rejects a version that already exists on crates.io with:

```
error: crate version `0.1.0` is already uploaded
```

You don't need to — the published version is correct and already on the registry.

## Verification step first

Confirm what actually landed on crates.io before doing anything:

```sh
cargo search kan-core          # check current published version
cargo info kan-core            # if cargo-info is installed
```

Or check https://crates.io/crates/kan-core directly.

## Resume procedure

### 1. Ensure the registry index has propagated

crates.io can take 1–2 minutes to make a new crate available for download. If
`kan-adapters` depends on the just-published `kan-core`, cargo needs to resolve
it from the registry. Wait ~2 minutes, then:

```sh
cargo update -p kan-core
```

### 2. Publish kan-adapters

```sh
cargo publish -p kan-adapters
```

If `kan-adapters` has a path+version dependency on `kan-core` (the standard
workspace pattern), cargo will resolve the registry version. Confirm
`Cargo.toml` for `kan-adapters` has:

```toml
[dependencies]
kan-core = { path = "../kan-core", version = "0.1.0" }
```

The `path` key is ignored by the registry; `version` is what gets recorded.
This is correct and required.

### 3. Publish kan-cli

After `kan-adapters` is live (again, allow ~1–2 min for propagation):

```sh
cargo publish -p kan-cli
```

### 4. Tag the release (if not yet done)

```sh
git tag -s v0.1.0 -m "Release v0.1.0"
git push origin v0.1.0
```

If the tag already existed pre-interruption, skip this.

## If kan-adapters publish also fails mid-flight

Same approach: check what's on crates.io, skip anything already published,
continue with the next unpublished crate in dependency order:

```
kan-core → kan-adapters → kan-cli
```

## Key rules

- Never bump the version to work around a partially published release — that
  creates a version mismatch between what's in git and what's on the registry.
- Never `--no-verify` unless the crate has a verified-clean state locally.
- Publish in strict dependency order; cargo will reject publishing a crate whose
  dependencies aren't yet on the registry.
