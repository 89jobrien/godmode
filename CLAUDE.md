# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this
repository.

Read local memory: @`.claude.local.md`

## Build & Test

```bash
cargo build --workspace
cargo check --workspace
cargo clippy --workspace -- -D warnings
cargo nextest run --workspace          # preferred
cargo test --workspace                 # fallback
cargo fmt --all --check                # format check
just conformance                       # plugin structure + subcommand + consistency checks

# Run a single test
cargo nextest run -E 'test(runnable_returns_tasks)'
cargo test -p godmode-core runnable_returns_tasks
```

## Install the CLI

```bash
cargo install --path crates/godmode-cli --root ~/.local
# binary lands at ~/.local/bin/godmode
```

## Architecture

Two-crate workspace:

- **`crates/godmode-core`** — library; all domain logic and integrations
- **`crates/godmode-cli`** — binary (`godmode`); thin clap CLI that calls into core

### Core modules (`godmode-core/src/`)

| Module          | Responsibility                                                                                                                                           |
| --------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `model`         | `Task`, `TaskGraph`, `Status` — the data model. `TaskGraph` serializes to `.ctx/GODMODE.tasks.yaml`.                                                     |
| `graph`         | Load/save task file, all task state transitions (`start`, `complete`, `block`, `unblock`, `add`, `remove`, `clear`), `runnable()` dependency resolution. |
| `detect`        | Walks up from CWD to find git root; reads `[package] name` from `Cargo.toml`.                                                                            |
| `plan`          | Parses plan markdown (`### Task N: <title>`) into `Task` structs with sequential deps.                                                                   |
| `dispatch`      | Groups tasks into independent crate-scoped chains for parallel agent dispatch.                                                                           |
| `session`       | `handoff()` — validates session-end state (counts running tasks).                                                                                        |
| `integrations/` | Thin subprocess wrappers: `doob` (todo sync), `hj` (handoff YAML), `rx` (run: field dispatch), `cruxx` (trace event append).                             |

### State file

`.ctx/GODMODE.tasks.yaml` — ephemeral, gitignored. Created automatically on first write.
`graph::load` returns an empty `TaskGraph` if the file is absent (no error).

### Trace events

`start_traced` / `complete_traced` append JSONL to `.ctx/GODMODE.trace.jsonl` via the cruxx
integration. All trace writes are non-fatal (`let _ = cruxx::append_event(...)`).

### Integration pattern

All integrations (`doob`, `hj`, `rx`, `cruxx`) invoke external binaries via `std::process::Command`.
They fail gracefully — callers use `.ok()` or `ok().flatten()` so missing tools don't abort the
session. Never add a hard dependency on an external tool; always degrade gracefully.

### `--json` flag

Every `godmode` subcommand accepts `--json` (global). Human-readable output goes to stdout by
default; `--json` emits machine-readable JSON for skill/agent consumers. Add `--json` support to
any new subcommand.

### Plan ingestion format

Plan markdown must use `### Task N: <title>` headings. Optionally annotate with:

```markdown
**Crate**: `crate-name`
**Run**: `cargo nextest run -p crate-name`
```

`plan::parse` builds sequential `depends_on` chains automatically. `graph::add` is idempotent —
re-ingesting a plan skips existing task IDs silently.

### CLI subcommands

```
godmode handon                          # session-start triage summary
godmode handoff                         # session-end validation
godmode status                          # graph counts + next runnable tasks
godmode task list / next / add / start / done / block / unblock / remove / clear
godmode task run <id> [--auto-done]     # execute task's run: field via rx
godmode task pull [--project <name>]    # import pending doob todos
godmode task push-done                  # sync completed tasks back to doob
godmode plan ingest <path>              # parse plan markdown into task graph
godmode dispatch [--max N]             # emit parallel chains JSON
godmode agent <path> [--max N]         # plan ingest + dispatch in one shot
```

`task done` accepts `--commit <sha>` and `--notes <text>` for trace metadata.
`task clear` requires `--done` (completed only) or `--all`.

## Plugin layout

This repo is also a Claude Code plugin installed via bazaar:

```
.claude-plugin/plugin.json   # name, version, author, description only — no extra fields
skills/                      # discovered by directory scan, not declared in plugin.json
agents/
```

Plugin manifest schema accepts only: `name`, `version`, `author`, `description`. Extra fields
cause validation failure on `claude plugin install`.
