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
