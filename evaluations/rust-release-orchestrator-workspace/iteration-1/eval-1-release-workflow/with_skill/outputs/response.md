# Releasing the `kan` Workspace to crates.io

The skill uses a Rust script at `~/.agents/skills/rust-release-orchestrator/scripts/orchestrate.rs` to handle dependency ordering, quality gates, publish retry, and resume. Here is the exact sequence.

---

## Prerequisites

Make sure you are authenticated with crates.io before starting:

```bash
cargo login
```

If that's already done, skip it.

---

## Step 1 — Dry run

Run this from the workspace root:

```bash
SCRIPT=~/.agents/skills/rust-release-orchestrator/scripts/orchestrate.rs
rust-script "$SCRIPT" --workspace /dev/kan --dry-run
```

**What this does:**

- Runs `cargo metadata` and topologically sorts the three crates by their dependency graph.
- Expected publish order: `kan-core` → `kan-adapters` → `kan-cli`
  - `kan-adapters` depends on `kan-core`, so `kan-core` must publish first.
  - `kan-cli` depends on both, so it publishes last.
- Runs quality gates in order: `cargo fmt --check`, `cargo clippy`, `cargo nextest run`.
  - If fmt fails, it auto-runs `cargo fmt --all` and re-checks.
  - If clippy fails, it auto-runs `cargo clippy --fix --allow-dirty --allow-staged` and re-checks. If still failing, it halts.
  - Test failures are NOT auto-fixed — you must fix them manually and re-run.
- Skips actual `cargo publish` — this is safe to run any number of times.

**Review the output:** Confirm the publish order is `kan-core → kan-adapters → kan-cli`. If any crate appears out of order, check its `[dependencies]` section for a missing workspace dep. Also check whether any crates have `publish = false` in their `Cargo.toml` — the script skips those silently.

---

## Step 2 — Confirm before publishing

Before running the real publish, confirm:

1. Publish order is correct: `kan-core` → `kan-adapters` → `kan-cli`
2. All gate checks passed (or auto-fixes were applied and checks passed)
3. Versions in each `Cargo.toml` are correct and haven't been published before
4. Any auto-fixes from fmt/clippy are committed

---

## Step 3 — Real run

```bash
rust-script "$SCRIPT" --workspace /dev/kan
```

This runs the same gates, then calls `cargo publish` for each crate in order with exponential-backoff retry:

- Max 5 attempts per crate
- Delays: 5s → 10s → 20s → 40s → 80s
- "already published" counts as success (safe to re-run)
- Rate-limit / 429 responses are retried automatically
- Any other failure halts after the retry limit

---

## Step 4 — If interrupted, resume

If the process is interrupted mid-run (network drop, Ctrl-C, etc.), the script writes `.release-state.json` in the workspace root tracking which crates already published. Resume with:

```bash
rust-script "$SCRIPT" --workspace /dev/kan --resume
```

Already-published crates are skipped. To start completely fresh, delete the state file first:

```bash
rm /dev/kan/.release-state.json
rust-script "$SCRIPT" --workspace /dev/kan
```

---

## Step 5 — Skip gates on resume (optional)

If gates already passed and you just need to retry a failed publish step:

```bash
rust-script "$SCRIPT" --workspace /dev/kan --resume --skip-gates
```

---

## Final output

The script emits a coloured table at the end showing each crate's status:

| Crate        | Status    | Elapsed |
| ------------ | --------- | ------- |
| kan-core     | published | ~Xs     |
| kan-adapters | published | ~Xs     |
| kan-cli      | published | ~Xs     |

Any `skipped` entries mean the version was already on crates.io. Any `failed` entries show the error and halt the run — fix the underlying issue, then `--resume`.

---

## Common issues for this workspace

| Symptom                                    | Fix                                                                                         |
| ------------------------------------------ | ------------------------------------------------------------------------------------------- |
| `kan-adapters` publishes before `kan-core` | Check `crates/kan-adapters/Cargo.toml` — `kan-core` must appear as a `[dependencies]` entry |
| Auth error on `cargo publish`              | Run `cargo login`                                                                           |
| Clippy still fails after auto-fix          | Fix manually, commit, then `--resume --skip-gates`                                          |
| State file from a previous failed run      | Delete `.release-state.json`, re-run from scratch                                           |
