# Baselines and scan scope

Read this before saving a baseline, wiring a CI gate, or comparing scores from a workspace scan and a per-crate scan.

## Baselines and CI Integration

Save a baseline for CI regression detection:

```bash
rustqual --init                                      # Generate tailored config
rustqual --save-baseline .ctx/rustqual-baseline.json # Snapshot current state
rustqual --compare .ctx/rustqual-baseline.json --fail-on-regression  # CI gate
```

This exits 0 as long as the score doesn't drop, even if findings exist.
Commit both `rustqual.toml` and `.ctx/rustqual-baseline.json` to the repo.

## Workspace vs Per-Crate Scanning

rustqual produces different totals depending on scan scope:

- **Workspace-level** (`rustqual .` from repo root): scans all `.rs` files
  including top-level re-exports and files not inside any single crate.
- **Per-crate** (`rustqual crates/<name>`): scans only that crate's `src/`.

The per-crate sum will be lower than the workspace total because the
workspace scan includes files outside any crate boundary. These numbers
are not comparable — always use the same method when tracking progress.

**Rule:** Use `--save-baseline` / `--compare` instead of recording numbers
in plan documents. The baseline file is the single source of truth.
