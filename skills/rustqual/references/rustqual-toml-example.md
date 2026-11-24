# rustqual.toml example

Read this before writing or editing `rustqual.toml`. Run `rustqual --init` first to get a tailored file, then compare it against this example for the keys and the weight block.

```toml
# Ignore functions by name pattern (glob). "main" and "test_*" are common.
ignore_functions = ["main", "test_*"]

# Skip entire files
exclude_files = []

# IOSP tuning
strict_closures = false           # Treat closures as logic
strict_iterator_chains = false    # Treat .map/.filter as logic
strict_error_propagation = false  # Count ? as logic

[complexity]
max_cognitive = 18
max_cyclomatic = 11
max_function_lines = 68

[duplicates]
detect_dead_code = true           # Set false for workspace crates
                                  # where pub API is used cross-crate

[srp]
max_parameters = 5
file_length_baseline = 300
file_length_ceiling = 800
lcom4_threshold = 2

[weights]
# Must sum to ~1.0. Adjust to match project priorities.
iosp         = 0.22
complexity   = 0.18
dry          = 0.13
srp          = 0.18
coupling     = 0.09
test_quality = 0.10
architecture = 0.10
```
