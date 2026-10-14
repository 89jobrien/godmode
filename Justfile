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

# Build release and install to the earliest ~/.local/bin or ~/.cargo/bin on PATH.
# Delegate to xtask rather than copying by hand: xtask honours CARGO_TARGET_DIR,
# and a hand-rolled `cp target/release/godmode` silently copies a stale binary
# whenever CARGO_TARGET_DIR points elsewhere.
install:
    cargo xtask install

# Run Rust tests
test:
    cargo nextest run --workspace --all-features

# Run CI gates (matches GitHub Actions)
ci: test
    cargo clippy --workspace -- -D warnings
    cargo fmt --all --check
