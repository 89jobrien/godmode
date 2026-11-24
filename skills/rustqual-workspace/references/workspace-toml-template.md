# Workspace rustqual.toml template

Read this before creating or editing the workspace-level `rustqual.toml`, or a per-crate override.

```toml
# Workspace-level config — place in repo root
ignore_functions = ["main", "run", "dispatch_*"]

[duplicates]
# Set false only if ALL dead-code findings are cross-crate pub API false positives.
# Leave true if any genuine dead code exists.
detect_dead_code = false

[complexity]
max_cognitive = 18
max_cyclomatic = 11
max_function_lines = 68

[srp]
max_parameters = 5
file_length_baseline = 300
file_length_ceiling = 800
lcom4_threshold = 2

[weights]
iosp         = 0.22
complexity   = 0.18
dry          = 0.13
srp          = 0.18
coupling     = 0.09
test_quality = 0.10
architecture = 0.10
```

For per-crate configs, place a `rustqual.toml` in each crate directory. Per-crate
configs override the workspace config for that crate's scan.
