# Resuming an Interrupted kan Workspace Release

## The command

```bash
cd /dev/kan
rust-script ~/.agents/skills/rust-release-orchestrator/scripts/orchestrate.rs --workspace . --resume
```

## Why it works

The orchestrator writes `.release-state.json` in the workspace root as it publishes each crate. This file records which crates have already been published successfully.

When you run with `--resume`, the script:

1. Loads `.release-state.json` from the workspace root
2. Skips any crate already marked as published (including `kan-core`)
3. Continues the topologically-sorted publish sequence from where it left off — so `kan-adapters` publishes next (it depends on `kan-core`), then `kan-cli` (which depends on both)

The "already published" guard also applies here: even if `kan-core`'s entry somehow got cleared from the state file, `cargo publish` would return an "already uploaded" error which the orchestrator treats as success — so re-running without `--resume` is also safe, just less efficient.

## If gates already passed before the interruption

Add `--skip-gates` to avoid re-running fmt/clippy/tests:

```bash
rust-script ~/.agents/skills/rust-release-orchestrator/scripts/orchestrate.rs --workspace . --resume --skip-gates
```

## If the state file is missing or stale

If `.release-state.json` was deleted or is from a different release attempt, run without `--resume`. The orchestrator will attempt to publish all crates; `kan-core` will get an "already uploaded" response which is treated as success, and it will proceed to `kan-adapters` and `kan-cli`.

```bash
rust-script ~/.agents/skills/rust-release-orchestrator/scripts/orchestrate.rs --workspace .
```
