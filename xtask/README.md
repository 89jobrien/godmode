# Workspace `xtask`

<!-- markdownlint-disable MD013 -->

`xtask` is the repository-local maintenance binary for the Godmode Rust workspace. It gives
developers and CI one implementation of the quality gates, distribution build, and local CLI
installation flow. The package is private (`publish = false`) and is not part of Godmode's runtime
API.

Run every command from the workspace root:

```console
cargo xtask <COMMAND>
```

Running `cargo xtask` without a command prints the supported command list. An unknown command or a
failed child process returns a non-zero exit status.

## Commands

| Command      | Purpose                     | Operations, in order                                     |
| ------------ | --------------------------- | -------------------------------------------------------- |
| `pre-commit` | Fast local gate             | format check, workspace clippy, typed conformance runner |
| `ci`         | Full repository gate        | all checks through release validation                    |
| `dist`       | Build the distributable CLI | release build of the `godmode-cli` package               |
| `install`    | Install the CLI locally     | `dist`, then copy `godmode` to `$HOME/.cargo/bin/`       |

`pre-commit` executes these commands:

```console
cargo fmt --all --check
cargo clippy --workspace -- -D warnings
cargo run -p godmode-conformance --bin run-conformance -- --verbose
```

`ci` is fail-fast and executes:

```console
cargo fmt --all --check
cargo check --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo nextest run --workspace --all-features
cargo run -p godmode-conformance --bin run-conformance -- --verbose
cargo deny check
cargo run -q -p godmode-cli -- release validate
```

The explicit typed conformance run is separate from `nextest`: it uses the custom report generator
and returns failure when any registered conformance case fails. See
`tests/conformance/README.md` for that package's role and output modes.

## Distribution And Installation

Build a release binary without copying it:

```console
cargo xtask dist
```

The binary is read from `<target-dir>/release/godmode`. `xtask` honors `CARGO_TARGET_DIR`; a
relative value is resolved from the workspace root. Without that variable, it uses the workspace
`target/` directory.

Build and install for the current user:

```console
cargo xtask install
```

Installation creates `$HOME/.cargo/bin/` when necessary and copies the release binary to
`$HOME/.cargo/bin/godmode`. `HOME` must be set.

## Implementation Contract

- Child commands always run with the workspace root as their working directory, even when the
  `xtask` binary is launched elsewhere.
- Gates stop at the first failed process and preserve that failure as a non-zero `xtask` result.
- The CI nextest invocation always includes both `--workspace` and `--all-features`.
- `dist` and `install` target the `godmode-cli` package, whose binary name is `godmode`.

These contracts are process-tested with fake executables in `xtask/tests/ci_contract.rs`; the unit
test in `xtask/src/main.rs` also pins the nextest argument list.

## Development And Testing

Run only the `xtask` tests:

```console
cargo nextest run -p xtask
```

The integration tests are Unix-specific because their fake tool fixtures create executable shell
scripts. To validate the complete repository contract, use:

```console
cargo xtask ci
```

When changing a gate, update the implementation, usage text, and process-level command-order
contract together. Keep `ci` fail-fast so later checks do not obscure the first actionable error.
