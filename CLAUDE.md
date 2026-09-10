# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this
repository, including the shared Codex and OpenCode projections.

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
cargo build --release -p godmode-cli && cp target/release/godmode ~/.cargo/bin/godmode
```

Note: `which godmode` resolves to `~/.cargo/bin/`, not `~/.local/bin/`. Always copy to
`~/.cargo/bin/` when rebuilding.

## Architecture

Four-member workspace:

- **`crates/godmode-core`** — library; all domain logic and integrations
- **`crates/godmode-cli`** — binary (`godmode`); thin clap CLI that calls into core
- **`tests/conformance`** — property, integration, and plugin contract tests
- **`xtask`** — local and CI quality-gate orchestration

### Core modules (`godmode-core/src/`)

| Module          | Responsibility                                                                                                                                                                                     |
| --------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `model`         | `Task`, `TaskGraph`, `Status` — the data model. `TaskGraph` serializes to `.ctx/godmode/tasks.yaml`.                                                                                               |
| `graph`         | Load/save task file, all task state transitions (`start`, `complete`, `block`, `unblock`, `add`, `remove`, `clear`), `runnable()` dependency resolution.                                           |
| `detect`        | Walks up from CWD to find git root; reads `[package] name` from `Cargo.toml`.                                                                                                                      |
| `plan`          | Parses plan markdown (`### Task N: <title>`) into `Task` structs with sequential deps.                                                                                                             |
| `dispatch`      | Groups tasks into independent crate-scoped chains for parallel agent dispatch.                                                                                                                     |
| `session`       | `Session` struct — owns all task transitions, duration tracking, crux trace writes, rx validation. `handon()`/`handoff()` are thin wrappers. `SessionSummary` emitted at handoff.                  |
| `integrations/` | External adapters for `doob`, `hj`, and `rx`, plus in-process Crux `Step` constructors; optional call sites choose their own degradation policy.                                                   |
| `templates`     | Template resolution, `{{var}}` substitution, apply to graph. Files in `templates/` or `~/.config/godmode/templates/`.                                                                              |
| `builder`       | Interactive (`graph build`) and file-driven graph construction. Phase logic: shape, wire, validate.                                                                                                |
| `verify`        | nextest + clippy + fmt + git log gate.                                                                                                                                                             |
| `wave`          | Parallel agent slot state — init, done, blocked, check.                                                                                                                                            |
| `worktree`      | Git worktree lifecycle — add (with GH issue link), remove.                                                                                                                                         |
| `workflow`      | Causal workflow DAGs per agent — YAML step definitions with `run:` and `depends_on` edges.                                                                                                         |
| `review`        | Plugin conformance auditing — checks skills, agents, and `plugin.json` for structural issues.                                                                                                      |
| `release`       | Version bump, annotated tag, push, and changelog generation from git commits since last tag.                                                                                                       |
| `skill`         | Skill registry — install/uninstall skills from local paths; persists to `~/.config/godmode/registry.json`.                                                                                         |
| `registry`      | `Registry` / `RegistryEntry` types; load/save `~/.config/godmode/registry.json`.                                                                                                                   |
| `agent_index`   | Regenerates `agents/INDEX.md` from `agents/cfg/` and `agents/*.md`.                                                                                                                                |
| `session_trace` | Low-level JSONL append helpers used by `session` for trace writes.                                                                                                                                 |
| `command`       | Loads canonical command YAML, renders Claude/OpenCode projections, checks drift, and writes or installs generated commands.                                                                        |
| `report_index`  | `ReportIndexPort` plus the JSON-file adapter for reconciling categorized report files under `.ctx/godmode/reports/`.                                                                               |
| `trace_stats`   | Reads observability JSONL and provides tail, failure, aggregate-statistics, and cross-session summary queries.                                                                                     |
| `config`        | Loads `.godmode.toml` (repo-local) or `~/.config/godmode/config.toml` (global fallback). Fields: `project_name`, `integrations` (doob/hj/rx toggles), `handoff` output settings.                   |
| `context`       | `SessionContext` struct — assembled by `godmode context [--json]`; exposes running tasks, blocked summary, recent commits, critical-path depth for hooks and subagents.                            |
| `cache`         | Writes `StatusCache` to `~/.cache/godmode/status.json` after every status update — designed for fast reads by starship prompt modules.                                                             |
| `agent`         | `AgentDef` / `AgentMetadata` / `AgentHook` types — parsed from `agents/cfg/*.cfg.yaml`; `generate_from_cfg` pairs with `agents/prompts/*.prompt.txt` to emit top-level `.md`.                      |
| `insights`      | Append-only JSONL insight capture (`.ctx/godmode/traces/insights.jsonl`). `append`, `list`, `list_for_date`, `render_markdown`. Bridges to `.ctx/godmode/reports/insights/insights-YYYY-MM-DD.md`. |
| `policy`        | Governance policy engine — loads, composes, and enforces agent policies from `skills/agent-governance/policies/`. Supports resolve, check, list, audit.                                            |
| `pipeline`      | Named multi-step skill sequences. State persists in `.ctx/godmode/pipeline.yaml`. Supports start, next, skip, stop, and status operations.                                                         |
| `testing`       | Feature-gated (`--features testing`) helpers: `audit`, `binary` (fake_bin), `conformance`, `env`, `prop`, `seed`. Used by the `godmode-conformance` workspace member only.                         |

### State file

`.ctx/godmode/tasks.yaml` — ephemeral, gitignored. Created automatically on first write.
`graph::load` returns an empty `TaskGraph` if the file is absent (no error).
Canonical task statuses are `pending`, `running`, `done`, and `blocked`. The parser accepts
legacy `active` as `running`, but serialization always emits `running`.

### Agent scratch space

`.ctx/godmode/_WORKING_DIR/` is the canonical scratch directory for all agents and helpers. Use it for:

- Intermediate proposals, plans, and drafts produced during a session
- `BLOCKED.md` files written by stalled parallel agents
- MoA proposal files (`moa-proposal-<n>.txt`)
- Any artifact that should survive within a session but need not be committed

**Naming convention**: `<agent-or-skill>-<artifact>.<ext>`
(e.g. `moa-proposal-1.txt`, `introspection-2026-05-02.md`, `parallel-blocked-crate-foo.md`)

`godmode handon` reports a count of files present. `godmode handoff` may snapshot keepers into
`.ctx/` proper. Everything in `_WORKING_DIR/` is gitignored via the `.ctx/*` rule.

### Trace events

`Session::start_task` / `Session::complete_task` append `crux_runtime::Step` JSONL to
`.ctx/godmode/sessions/YYYY-MM-DD.jsonl`. `Session::handoff` writes a `SessionSummary` record to
`.ctx/godmode/sessions/YYYY-MM-DD-summary.jsonl`. All trace writes are non-fatal (`let _ = ...`).

### Integration pattern

`doob` and `hj` are subprocess adapters whose public functions return errors when binaries or
commands fail; selected session and sync call sites deliberately treat those integrations as
best-effort. `rx` skips registry validation only when `rx` is absent, but rejects missing registered
scripts when it is available. Crux task traces use the in-process `crux-runtime` dependency, and
trace-file writes are best-effort. Do not assume every integration failure is swallowed; degrade
gracefully only at call sites where the operation is explicitly optional.

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

This inventory is derived from `godmode --help` and each command family’s `--help`; use
`godmode <command> --help` for action-specific arguments.

```text
godmode handon
godmode handoff
godmode session {prune}
godmode trace {tail|failures|stats|summary}
godmode task {list|add|start|done|block|unblock|remove|clear|next|run|pull|push-done|unblock-all|apply|list-templates}
godmode plan {ingest}
godmode command {generate|install-opencode}
godmode dispatch
godmode context
godmode status
godmode agent {list|index|dispatch|generate|install-opencode|migrate}
godmode verify
godmode wave {init|status|done|block|check}
godmode worktree {add|remove}
godmode ci {triage}
godmode issue {list|close}
godmode graph {build}
godmode hook {list|log|test|migrate|run}
godmode skill {list|index|install|uninstall}
godmode review {self|skills|agents}
godmode release {current|bump|tag|push|changelog|validate}
godmode workflow {run|list|status}
godmode visualize-graph
godmode memory-banking {inject|remind|init|status}
godmode insight {add|list|render}
godmode pipeline {list|show|start|next|skip|stop|status|run}
godmode policy {resolve|check|list|audit}
godmode pin
godmode unpin
godmode init
godmode doctor
godmode scaffold
godmode test-check
```

`task done` accepts `--commit <sha>` and `--notes <text>` for trace metadata.

### Pipeline

```
godmode pipeline list                           # show all pipelines
godmode pipeline show <name>                    # show steps with current position
godmode pipeline start <name> [--from <skill>]  # start and auto-invoke first step
godmode pipeline next                           # advance and invoke next step
godmode pipeline skip                           # advance without invoking
godmode pipeline stop                           # deactivate pipeline, preserve state
godmode pipeline status                         # show active pipeline + position
godmode pipeline run <name> [--from <skill>] [--fail-fast]  # headless: walk task graph, execute run: fields
```

Nine pipelines are defined in `pipelines/`:

- `feature` — idea to merged PR
- `parallel-feature` — fan-out across crates
- `release` — health check, audit, changelog, tag
- `maintenance` — health scorecard and targeted cleanup
- `triage` — issue backlog to task graph
- `retrospective` — session reflection and learning
- `lifecycle` — full session from handon to handoff
- `aichat-system` — aichat system prompt generation and installation
- `coursers-rules` — Coursers rule discovery, validation, and installation

Pipeline state persists in `.ctx/godmode/pipeline.yaml` (gitignored).
`task clear` requires `--done` (completed only) or `--all`.

## Plugin layout

This repo is also a Claude Code plugin installed via bazaar:

```
.claude-plugin/plugin.json   # name, version, author, description only — no extra fields
skills/                      # discovered by directory scan, not declared in plugin.json
agents/                      # top-level *.md are GENERATED — Claude discovers these
  cfg/*.cfg.yaml             # source of truth: structured agent config
  prompts/*.prompt.txt       # source of truth: raw prompt text
  INDEX.md                   # generated: agent table
```

Plugin manifest schema accepts only: `name`, `version`, `author`, `description`. Extra fields
cause validation failure on `claude plugin install`.

## Gotchas

- CLI Quick Reference (`## CLI subcommands`) can silently drift from `crates/godmode-cli/src/main.rs`
  — when adding/changing a `Cmd` variant, grep `enum.*Action` in `main.rs` and diff against the
  reference block. `skill`/`release`/`pipeline`/`policy` families were undocumented for a while.
- `skills/introspection/helpers/audit.nu` checks skill-index completeness and cross-references
  subcommand calls; run it after editing any `skills/*/SKILL.md` or `agents/*.md`. Report lands in
  `.ctx/godmode/reports/introspection/` (gitignored).
- `godmode plan ingest` skips tasks whose IDs already exist — plans reuse `t1`/`t2`/etc.
  If ingesting multiple plans into one graph, add tasks manually with distinct IDs.
- `godmode task add <title> --id <id> --depends-on ""` registers an empty string as a dep,
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

## Rust Conventions

- Run `cargo check --workspace` before committing.
- Fix clippy warnings proactively — treat `-D warnings` as the standard.
- Run `cargo test` (or `cargo nextest run`) if test files were modified.
- Do not investigate rust-analyzer or IDE diagnostics unless explicitly asked — they are often
  stale.

## CI

List the latest runs on the current branch without opening a TTY watcher:

```nu
gh run list --branch (git branch --show-current) --limit 3
```

```bash
gh run list --branch $(git branch --show-current) --limit 3
```

## Git Operations

- NEVER use `--no-verify` on git commits. Always let pre-commit hooks run.
- Before claiming a branch is merged, verify with `git log --oneline main..branch` — empty
  output means fully merged.
- Never drop git stashes without showing the diff and getting explicit confirmation.
- Scope staged changes precisely to the current task. Do not stage unrelated changes.

## Subagent Guardrails

When dispatching subagents:

- Each subagent must run `git branch --show-current` immediately before every `git commit`.
  If the answer is `main`, STOP — do not commit to main directly.
- Worktree subagents MUST merge their branch back and remove the worktree before reporting done.
  An orphaned worktree means the task is incomplete.
- After subagents complete, verify their changes were committed (`git log --oneline -3`).
  A HANDOFF with `commits: []` is incomplete.
- Never use octopus merges across subagents — cherry-pick sequentially if branches diverge.
- Cap parallel subagents at 5 concurrent to avoid API rate limits.
- Never use `--no-verify` in subagent git operations.
- If tests fail, debug and retry up to 3 times before escalating.

## Sentinel Reviews

Apply ALL severity levels (blocking, suggestion, nitpick) in one pass before committing.
Do not commit after fixing only blocking issues and leave suggestions for a follow-up — that
creates noisy multi-pass fix histories. One sentinel run, one fix commit.

## Nushell

Hook scripts in this repo are Nushell. Key syntax rules:

- `const` cannot reference `$env` — use `let` or read at runtime with `$env.VAR`
- `&&` is not valid — use `;` to chain commands
- `open --raw /dev/stdin | from json` to read stdin in hook scripts
- `do { ... } | complete` captures stdout + exit code for fallible commands
- Never use bash-isms: no `$()`, no `export VAR=val`, no `if [ ... ]`
- Test syntax with `nu -c '<snippet>'` before writing to a file

## Output Style

- No superlatives in generated output. Do not use "impressive", "beautifully", "remarkable",
  "industrial-scale", or similar inflated language. State facts plainly.
- No emojis unless explicitly requested.
- No sycophantic openers ("Great question!", "Absolutely!").
- Act first, explain later. When a task is clear, do it — don't narrate the approach first.
