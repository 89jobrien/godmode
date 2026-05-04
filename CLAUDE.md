# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this
repository.

Read local memory: @`.claude.local.md`

## Build & Test

```bash
cargo run -p godmode-conformance --bin run-conformance -- --verbose  # run conformance suite
cargo test -p godmode-conformance                                     # property tests
cargo bench -p godmode-conformance                                    # criterion benchmarks
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

| Module          | Responsibility                                                                                                                                                                     |
| --------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `model`         | `Task`, `TaskGraph`, `Status` — the data model. `TaskGraph` serializes to `.ctx/GODMODE.tasks.yaml`.                                                                               |
| `graph`         | Load/save task file, all task state transitions (`start`, `complete`, `block`, `unblock`, `add`, `remove`, `clear`), `runnable()` dependency resolution.                           |
| `detect`        | Walks up from CWD to find git root; reads `[package] name` from `Cargo.toml`.                                                                                                      |
| `plan`          | Parses plan markdown (`### Task N: <title>`) into `Task` structs with sequential deps.                                                                                             |
| `dispatch`      | Groups tasks into independent crate-scoped chains for parallel agent dispatch.                                                                                                     |
| `session`       | `Session` struct — owns all task transitions, duration tracking, cruxx trace writes, rx validation. `handon()`/`handoff()` are thin wrappers. `SessionSummary` emitted at handoff. |
| `integrations/` | Thin subprocess wrappers: `doob` (todo sync), `hj` (handoff YAML), `rx` (run: dispatch + `list_scripts`/`validate_run`), `cruxx` (Step constructors).                              |
| `templates`     | Template resolution, `{{var}}` substitution, apply to graph. Files in `templates/` or `~/.config/godmode/templates/`.                                                              |
| `builder`       | Interactive (`graph build`) and file-driven graph construction. Phase logic: shape, wire, validate.                                                                                |
| `verify`        | nextest + clippy + fmt + git log gate.                                                                                                                                             |
| `wave`          | Parallel agent slot state — init, done, blocked, check.                                                                                                                            |
| `worktree`      | Git worktree lifecycle — add (with GH issue link), remove.                                                                                                                         |
| `workflow`      | Causal workflow DAGs per agent — YAML step definitions with `run:` and `depends_on` edges.                                                                                         |
| `review`        | Plugin conformance auditing — checks skills, agents, and `plugin.json` for structural issues.                                                                                      |
| `release`       | Version bump, annotated tag, push, and changelog generation from git commits since last tag.                                                                                       |
| `skill`         | Skill registry — install/uninstall skills from local paths; persists to `~/.config/godmode/registry.json`.                                                                         |
| `registry`      | `Registry` / `RegistryEntry` types; load/save `~/.config/godmode/registry.json`.                                                                                                   |
| `agent_index`   | Regenerates `agents/INDEX.md` from agent YAML/MD files in `agents/`.                                                                                                               |
| `session_trace` | Low-level JSONL append helpers used by `session` for trace writes.                                                                                                                 |

### State file

`.ctx/GODMODE.tasks.yaml` — ephemeral, gitignored. Created automatically on first write.
`graph::load` returns an empty `TaskGraph` if the file is absent (no error).

### Agent scratch space

`.ctx/_WORKING_DIR/` is the canonical scratch directory for all agents and helpers. Use it for:

- Intermediate proposals, plans, and drafts produced during a session
- `BLOCKED.md` files written by stalled parallel agents
- MoA proposal files (`moa-proposal-<n>.txt`)
- Any artifact that should survive within a session but need not be committed

**Naming convention**: `<agent-or-skill>-<artifact>.<ext>`
(e.g. `moa-proposal-1.txt`, `introspection-2026-05-02.md`, `parallel-blocked-crate-foo.md`)

`godmode handon` reports a count of files present. `godmode handoff` may snapshot keepers into
`.ctx/` proper. Everything in `_WORKING_DIR/` is gitignored via the `.ctx/*` rule.

### Trace events

`Session::start_task` / `Session::complete_task` append `cruxx_core::Step` JSONL to
`.ctx/sessions/YYYY-MM-DD.jsonl`. `Session::handoff` writes a `SessionSummary` record to
`.ctx/sessions/YYYY-MM-DD-summary.jsonl`. All trace writes are non-fatal (`let _ = ...`).

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
godmode handon                                  # session-start triage summary
godmode handoff                                 # session-end validation
godmode status [--compact]                      # graph counts + next runnable tasks
godmode task list [--priority high|normal|low]
godmode task next [--priority high|normal|low]
godmode task add [<id>] <title> [--depends-on t1,t2] [--crate-name X]
godmode task start <id>
godmode task done <id> [--commit <sha>] [--notes <text>]
godmode task block <id> <reason>
godmode task unblock <id>
godmode task unblock-all                        # reset all blocked tasks to pending
godmode task remove <id>
godmode task clear --done | --all
godmode task run <id> [--auto-done]             # execute task's run: field via rx
godmode task pull [--project <name>]            # import pending doob todos
godmode task pull --github [--repo owner/repo] [--label <label>]
godmode task apply <name> [--var k=v]           # expand a template into the task graph
godmode task list-templates                     # list available templates (local + global)
godmode task push-done                          # sync completed tasks back to doob
godmode plan ingest <path>                      # parse plan markdown into task graph
godmode dispatch [--max N] [--critical-path]    # emit parallel chains JSON
godmode agent list [--filter <kw>]              # list installed agents
godmode agent index                             # regenerate agents/INDEX.md
godmode agent dispatch <path> [--max N]         # plan ingest + dispatch in one shot
godmode agent generate [<name>] [--all]         # generate .md from agent YAML
godmode agent migrate [<name>] [--all]          # migrate agent .md frontmatter to YAML stubs
godmode graph build [--input <tmpl>] [--var k=v]
godmode verify [--crate-name X]                 # nextest + clippy + fmt + commits
godmode wave init --wave N --agents a,b,c
godmode wave status / done <agent> / block <agent> / check
godmode worktree add <branch> [--issue N]
godmode worktree remove <branch>
godmode ci triage [--run-id <id>]
godmode issue list [--repo owner/repo] [--label <label>]
godmode issue close <number> --commit <sha> [--repo owner/repo]
godmode hook list / log [--tail N] / test <script> / migrate
godmode skill list / install <path> / uninstall <name>
godmode review self / skills / agents
godmode release current / bump [--version X] / tag / push / changelog
godmode session prune --older-than <days> [--dry-run]
godmode workflow run <agent> <workflow>
godmode workflow list [--agent <name>]
godmode workflow status <name>
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

## Gotchas

- `godmode plan ingest` skips tasks whose IDs already exist — plans reuse `t1`/`t2`/etc.
  If ingesting multiple plans into one graph, add tasks manually with distinct IDs.
- `godmode task add <id> <title> --depends-on ""` registers an empty string as a dep,
  causing "unmet dependencies" on start. Omit `--depends-on` entirely for root tasks.
- `dispatch --critical-path` shows the critical path tasks; `godmode status` also surfaces it.
- Pre-commit hook runs `cargo fmt` automatically — expect a format diff on first commit attempt.
- `plan::parse` returns `Result<Vec<Task>>`, not `Vec<Task>` — always match/unwrap the Result.
- `dispatch::independent_chains(graph, max)` returns `Vec<Chain>` — not `build_slots`.
- `cargo fmt` PostToolUse hook runs automatically but does NOT auto-stage; run `cargo fmt --all`
  then `git add` again before committing or the pre-commit check will still fail.
- `tests/conformance/` is a workspace member (`-p godmode-conformance`); add new test modules
  in `src/`, register in `lib.rs::all_tests()`, and add `pub mod` to `lib.rs`.
- `Task::started_at` is set by `Session::start_task`, not `graph::start` — duration tracking
  only works when transitions go through `Session`, not raw `graph::*` functions directly.
- `rx::validate_run` fires inside `Session::start_task` before state mutation — if the script
  doesn't exist and `rx` is on PATH, the task is rejected before being marked Running.
