# Crux pipelines

These `.crux` files are executable validation gates for the Crux CLI. They are separate from the
Godmode skill-sequencing YAML files in the parent directory.

## Gates

| Pipeline          | Description                                                                            |
| ----------------- | -------------------------------------------------------------------------------------- |
| `check_refs.crux` | Runs `scripts/check-refs.nu --ci` and fails when skill documentation references break. |

## Run

From the Godmode repository root, with the `crux` binary on `PATH`:

```bash
crux run pipelines/crux/check_refs.crux
```

To run against a sibling Crux checkout instead:

```bash
cargo run --quiet --manifest-path ../crux/Cargo.toml -p crux-cli --bin crux -- \
  run pipelines/crux/check_refs.crux
```
