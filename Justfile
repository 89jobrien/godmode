# Justfile for godmode

# Run the full conformance suite (plugin structure + subcommand + consistency checks)
conformance:
    nu tests/conformance/plugin-structure.nu

# Build all workspace crates (debug)
build:
    cargo build --workspace

# Build the CLI binary in release mode
build-release:
    cargo build --release -p godmode-cli

# Build release and install to ~/.cargo/bin
install: build-release
    cp target/release/godmode ~/.cargo/bin/godmode

# Run Rust tests
test:
    cargo nextest run --workspace

# Run CI gates (matches GitHub Actions)
ci: test
    cargo clippy --workspace -- -D warnings
    cargo fmt --all --check

# Run the check-refs crux pipeline gate (requires the sibling ../crux checkout)
crux-check-refs:
    cargo run --quiet --manifest-path ../crux/Cargo.toml -p crux-agentic --bin crux -- \
        run pipelines/crux/check_refs.crux -v
