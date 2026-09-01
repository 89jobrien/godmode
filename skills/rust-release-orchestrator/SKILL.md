---
name: rust-release-orchestrator
description: >
  Autonomous release orchestrator for Rust workspaces that publishes crates to
  crates.io. Use whenever the user says "publish crates", "release to crates.io",
  "release the workspace", "ship the crates", or wants to coordinate a Rust
  workspace release. Also trigger when the user asks about automating the release
  pipeline, handling publish order across multiple crates, or retrying failed
  crates.io uploads. This skill handles everything end-to-end: dependency ordering,
  gate validation, publish retry, and a final release report — so reach for it
  any time the topic is Rust crate publishing, even if the user doesn't say "skill".
---

# Rust Release Orchestrator

## What this skill does

Runs a bundled Python script (`scripts/orchestrate.py`) that:

1. **Computes publish order** — parses `cargo metadata` and topologically sorts workspace crates so dependencies publish before dependents
2. **Runs quality gates** — `cargo fmt`, `cargo clippy`, `cargo nextest run`, with auto-fix for fmt/clippy
3. **Publishes each crate** — `cargo publish` with exponential-backoff retry; treats "already published" as success and rate-limit responses as retryable
4. **Resumes cleanly** — writes `.release-state.json`; re-run with `--resume` after any interruption
5. **Emits a report** — coloured table of published / skipped / failed crates with elapsed time

## Workflow

Always run **dry-run first**, then real execution. Follow these steps:

### Step 1 — Locate the script

```bash
SCRIPT=~/.claude/skills/rust-release-orchestrator/scripts/orchestrate.py
```

### Step 2 — Dry run

```bash
python "$SCRIPT" --workspace . --dry-run
```

Review the printed publish order and gate results. Point out anything surprising to the user (unexpected crates, wrong order, gate failures that auto-fixed).

### Step 3 — Confirm with user

Tell the user what will publish, in what order, and whether any auto-fixes were applied. Ask for explicit confirmation before step 4.

### Step 4 — Real run

```bash
python "$SCRIPT" --workspace .
```

### Step 5 — If interrupted, resume

```bash
python "$SCRIPT" --workspace . --resume
```

The state file (`.release-state.json`) records which crates already published. Delete it to start fresh.

### Step 6 — Report to user

Paste the final report table from the script output. Note any skipped (already-published) or failed crates.

## Flags reference

| Flag                 | Effect                                                  |
| -------------------- | ------------------------------------------------------- |
| `--dry-run`          | Skip actual `cargo publish`, run everything else        |
| `--resume`           | Load `.release-state.json` and skip already-done work   |
| `--skip-gates`       | Bypass fmt/clippy/test (use after gates already passed) |
| `--workspace <path>` | Override workspace root (default: `.`)                  |

## What auto-fix covers

- **fmt**: runs `cargo fmt --all` if the check fails
- **clippy**: runs `cargo clippy --fix --allow-dirty --allow-staged` then re-checks; if still failing, halts

Auto-fix does NOT apply to test failures. Those must be fixed manually.

## Retry logic

- Max 5 attempts per crate
- Initial delay: 5 seconds, doubles each attempt (5 → 10 → 20 → 40 → 80s)
- "already exists" / "already uploaded": treated as success, no retry needed
- "too many requests" / "rate limit" / "429": retried with backoff
- Any other failure: retried up to the limit, then halts

## Common issues

| Symptom                                      | Fix                                                                                        |
| -------------------------------------------- | ------------------------------------------------------------------------------------------ |
| Wrong publish order                          | Check `[dependencies]` sections — a crate missing a workspace dep will be sorted too early |
| Gate fails after auto-fix                    | Clippy error requires manual fix; check the output, fix, then `--resume --skip-gates`      |
| `cargo publish` auth error                   | Run `cargo login` before starting                                                          |
| Interrupted mid-run                          | Re-run with `--resume`; already-published crates are skipped                               |
| State file stale from a previous botched run | Delete `.release-state.json` and re-run from scratch                                       |
| Private / path-only crates included          | Add `publish = false` to their `Cargo.toml`                                                |

## Notes on publishable crates

The script skips crates with `publish = false` in their `Cargo.toml`. Workspace members without an explicit `publish` field are assumed publishable. If the workspace has crates that shouldn't publish, set `publish = false` in their manifest before running.
