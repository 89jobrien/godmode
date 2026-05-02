# Justfile for godmode

# Run the full conformance suite (plugin structure + subcommand + consistency checks)
conformance:
    nu tests/conformance/plugin-structure.nu

# Run Rust tests
test:
    cargo nextest run --workspace

# Run CI gates (matches GitHub Actions)
ci: test
    cargo clippy --workspace -- -D warnings
    cargo fmt --all --check
