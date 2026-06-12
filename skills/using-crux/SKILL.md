---
name: "using-crux"
description: >
  Working with the crux codebase — an agentic Rust DSL and runtime trace model.
  Use when navigating crux source code, building crux from source, adding crux
  as a dependency, understanding crate boundaries, or debugging crux compilation
  issues. Triggers on "crux crate", "crux workspace", "crux build", "add crux
  dependency", "crux types", "CruxCtx", "Crux<T>", or any work inside the
  crux repo or a repo that depends on crux crates. Also use when someone asks
  about crux naming — the project was renamed from cruxx-* to crux-* in May 2026.
---

# Using Crux

Crux is an agentic DSL for Rust — macros, traits, and types that make agentic
control flow explicit in the type system. Every step, delegation, speculation,
and failure is a first-class value (`Crux<T>`) that is inspectable,
serializable, and replayable.

- **Edition**: Rust 2024
- **MSRV**: 1.88
- **License**: MIT
- **Repo**: `https://github.com/89jobrien/crux`

## Workspace Layout

All crates live under `crates/`. The workspace version is `0.2.6`.

| Crate          | Role                                                                 |
| -------------- | -------------------------------------------------------------------- |
| `crux`         | Facade — re-exports `crux-runtime` + `crux-macros`                   |
| `crux-runtime` | Core logic: types, traits, runtime, orchestrator types               |
| `crux-macros`  | Proc macros: `#[crux::agent]`, `#[crux::harness]`, `#[crux::evolve]` |
| `crux-types`   | Wire-format types (`Crux<T>`, `Step`, `Budget`, `CruxId`, `CruxErr`) |
| `crux-script`  | YAML pipeline runner (`.crux` files)                                 |
| `crux-agentic` | Step handlers (shell, fs, git, json, llm, container, etc.)           |
| `crux-plugin`  | Subprocess plugin host for pipelines                                 |
| `crux-planner` | Metrics-driven harness profile evolution                             |
| `crux-model`   | Canonical model ID types and provider parsers                        |
| `crux-domain`  | Domain vocabulary types                                              |
| `crux-baml`    | BAML structured extraction handlers                                  |
| `crux-stdlib`  | Standard library handlers (json, text, etc.)                         |
| `crux-improve` | Self-improvement and analysis handlers                               |

## Naming History

Crates were renamed in May 2026:

| Old name       | Current name   |
| -------------- | -------------- |
| `cruxx-core`   | `crux-runtime` |
| `cruxx-types`  | `crux-types`   |
| `cruxx-script` | `crux-script`  |
| `cruxx-plugin` | `crux-plugin`  |
| `cruxx-domain` | `crux-domain`  |

The facade crate package name is `cruxx` (for crates.io), but the lib name
is `crux`. Use `use crux::prelude::*;` in downstream code.

`slashcrux` is a separate vocabulary crate (`Priority`, `Urgency`,
`ExecutionContext`, `StepState`) — it was **not** renamed.

## Feature Flags

| Flag            | Default | Effect                                             |
| --------------- | ------- | -------------------------------------------------- |
| `tokio-runtime` | yes     | Async support (required for compilation)           |
| `redb`          | no      | `RedbBackend` — pure-Rust embedded KV store        |
| `tracing`       | no      | Instrument with tracing spans                      |
| `baml`          | no      | BAML structured extraction handlers (crux-agentic) |

## Build Commands

```bash
just ci              # Full gate: fmt + clippy + nextest
just test            # cargo nextest run
just lint            # cargo clippy --all-targets -- -D warnings
just fmt             # cargo fmt --all -- --check
just fix             # cargo fmt --all (in-place)
just build           # cargo build --all-targets
just check-baml      # Validate BAML version parity
```

Always use `cargo nextest run` instead of `cargo test`.

## Adding Crux as a Dependency

For **external consumers** that only need the wire types (e.g., minibox):

```toml
[dependencies]
crux-types = { git = "https://github.com/89jobrien/crux", rev = "<sha>" }
```

For **full runtime access**:

```toml
[dependencies]
cruxx = { git = "https://github.com/89jobrien/crux", rev = "<sha>" }
```

For **plugin protocol** (subprocess plugins):

```toml
[dependencies]
crux-plugin = { git = "https://github.com/89jobrien/crux", rev = "<sha>" }
```

## Key Types and Traits

See `references/types-and-traits.md` for the full reference with signatures
and usage examples.

## Architecture

See `references/architecture.md` for the hexagonal design, replay system,
and SOLID decomposition.

## Pipeline Files

Pipeline definitions use the `.crux` file extension (YAML syntax). See the
`planning-with-crux` skill for pipeline authoring guidance, or
`references/capabilities.md` for the full handler reference.

## Testing

Integration tests live in `crates/crux/tests/`. Test a single crate:

```bash
cargo nextest run -p crux-runtime
cargo nextest run -p crux --test agent_macro
cargo nextest run --features redb  # Include redb adapter tests
```

BAML tests require API keys — inject via:

```bash
DOTENV_PRIVATE_KEY=$(op read "op://Personal/<uuid>/password") \
  dotenvx run --env-file=$HOME/dev/.env -- \
  cargo nextest run --features baml -p crux-agentic
```
