# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this
repository.

Read local memory: @`.claude.local.md`

## Build & Test

All gates go through `taskit` — never invoke `cargo` directly.

```bash
taskit check quick                          # fast loop: fmt-check + lint + affected-crate tests
taskit check ci                             # full local CI, summary table
taskit check lint                           # clippy, -D warnings, all targets
taskit check lint --crate-name godmode-core # one crate
taskit check fmt --check                    # format check
taskit check fmt                            # format
taskit check compile                        # compile test binaries without running
taskit test run                             # nextest, workspace
taskit test run --crate-name godmode-core   # one crate
taskit test coverage                        # coverage with threshold
taskit test proptest --crate-name godmode-conformance
taskit test bench --crate-name godmode-conformance
taskit protocol audit                       # cargo-deny: advisories, licenses, bans
taskit protocol freshness                   # outdated dependency check
taskit check pre-commit                     # what the pre-commit hook runs

just conformance                            # plugin structure + subcommand + consistency checks
```

`taskit` auto-discovers the workspace from `Cargo.toml`, so no `taskit.toml` is required.
This table is a reference for ad-hoc runs — do not run these before committing or pushing,
because the global git hooks already gate format, clippy, and tests.

### Commands with no taskit equivalent

Keep `cargo` for these — taskit exposes no wrapper, and neither is a quality gate:

```bash
# Binary execution
cargo run -p godmode-conformance --bin run-conformance -- --verbose   # conformance suite

# Single-test filter (no passthrough on `taskit test run`)
cargo nextest run -E 'test(runnable_returns_tasks)'
cargo test -p godmode-core runnable_returns_tasks
```

## Install the CLI

```bash
taskit dev build --release && cp target/release/godmode ~/.cargo/bin/godmode
```

Note: `which godmode` resolves to `~/.cargo/bin/`, not `~/.local/bin/`. Always copy to
`~/.cargo/bin/` when rebuilding.

## Architecture

Two-crate workspace:

- **`crates/godmode-core`** — library; all domain logic and integrations
- **`crates/godmode-cli`** — binary (`godmode`); thin clap CLI that calls into core

### Core modules (`godmode-core/src/`)

| Module          | Responsibility                                                                                                                                                                            |
| --------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `model`         | `Task`, `TaskGraph`, `Status` — the data model. `TaskGraph` serializes to `.ctx/godmode/tasks.yaml`.                                                                                      |
| `graph`         | Load/save task file, all task state transitions (`start`, `complete`, `block`, `unblock`, `add`, `remove`, `clear`), `runnable()` dependency resolution.                                  |
| `detect`        | Walks up from CWD to find git root; reads `[package] name` from `Cargo.toml`.                                                                                                             |
| `plan`          | Parses plan markdown (`### Task N: <title>`) into `Task` structs with sequential deps.                                                                                                    |
| `dispatch`      | Groups tasks into independent crate-scoped chains for parallel agent dispatch.                                                                                                            |
| `session`       | `Session` struct — owns all task transitions, duration tracking, crux trace writes, rx validation. `handon()`/`handoff()` are thin wrappers. `SessionSummary` emitted at handoff.         |
| `integrations/` | Thin subprocess wrappers: `doob` (todo sync), `hj` (handoff YAML), `rx` (run: dispatch + `list_scripts`/`validate_run`), `crux` (Step constructors).                                      |
| `templates`     | Template resolution, `{{var}}` substitution, apply to graph. Files in `templates/` or `~/.config/godmode/templates/`.                                                                     |
| `builder`       | Interactive (`graph build`) and file-driven graph construction. Phase logic: shape, wire, validate.                                                                                       |
| `verify`        | nextest + clippy + fmt + git log gate.                                                                                                                                                    |
| `wave`          | Parallel agent slot state — init, done, blocked, check.                                                                                                                                   |
| `worktree`      | Git worktree lifecycle — add (with GH issue link), remove.                                                                                                                                |
| `workflow`      | Causal workflow DAGs per agent — YAML step definitions with `run:` and `depends_on` edges.                                                                                                |
| `review`        | Plugin conformance auditing — checks skills, agents, and `plugin.json` for structural issues.                                                                                             |
| `release`       | Version bump, annotated tag, push, and changelog generation from git commits since last tag.                                                                                              |
| `skill`         | Skill registry — install/uninstall skills from local paths; persists to `~/.config/godmode/registry.json`.                                                                                |
| `registry`      | `Registry` / `RegistryEntry` types; load/save `~/.config/godmode/registry.json`.                                                                                                          |
| `agent_index`   | Regenerates `agents/INDEX.md` from `agents/cfg/` and `agents/*.md`.                                                                                                                       |
| `session_trace` | Low-level JSONL append helpers used by `session` for trace writes.                                                                                                                        |
| `config`        | Loads `.godmode.toml` (repo-local) or `~/.config/godmode/config.toml` (global fallback). Fields: `project_name`, `integrations` (doob/hj/rx toggles), `handoff` output settings.          |
| `context`       | `SessionContext` struct — assembled by `godmode context [--json]`; exposes running tasks, blocked summary, recent commits, critical-path depth for hooks and subagents.                   |
| `cache`         | Writes `StatusCache` to `~/.cache/godmode/status.json` after every status update — designed for fast reads by starship prompt modules.                                                    |
| `agent`         | `AgentDef` / `AgentMetadata` / `AgentHook` types — parsed from `agents/cfg/*.cfg.yaml`; `generate_from_cfg` pairs with `agents/prompts/*.prompt.txt` to emit top-level `.md`.             |
| `insights`      | Append-only JSONL insight capture (`.ctx/godmode/traces/insights.jsonl`). `append`, `list`, `list_for_date`, `render_markdown`. Bridges to `.ctx/godmode/reports/insights-YYYY-MM-DD.md`. |
| `policy`        | Governance policy engine — loads, composes, and enforces agent policies from `skills/agent-governance/policies/`. Supports resolve, check, list, audit.                                   |
| `pipeline`      | Named multi-step skill sequences. State persists in `.ctx/godmode/pipeline.yaml`. Supports start, next, skip, stop, and status operations.                                                |
| `testing`       | Feature-gated (`--features testing`) helpers: `audit`, `binary` (fake_bin), `conformance`, `env`, `prop`, `seed`. Used by the `godmode-conformance` workspace member only.                |

### State file

`.ctx/godmode/tasks.yaml` — ephemeral, gitignored. Created automatically on first write.
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

`Session::start_task` / `Session::complete_task` append `crux_core::Step` JSONL to
`.ctx/godmode/sessions/YYYY-MM-DD.jsonl`. `Session::handoff` writes a `SessionSummary` record to
`.ctx/godmode/sessions/YYYY-MM-DD-summary.jsonl`. All trace writes are non-fatal (`let _ = ...`).

### Integration pattern

All integrations (`doob`, `hj`, `rx`, `crux`) invoke external binaries via `std::process::Command`.
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

`plan::parse` builds sequential `depends_on` chains automatically. The CLI prepares parsed tasks
against the current graph: the first plan keeps `tN` IDs, later collisions use a deterministic
file-stem namespace, and provenance keeps re-ingestion idempotent. The recorded path is one signal,
not the whole rule. A plan whose recorded path no longer exists on disk is matched as relocated
when its file stem is unambiguous and its recorded tasks are still in the graph with matching title,
crate, and run. That match keeps the existing task IDs and rebinds the provenance entry to the new
path, so re-ingestion is idempotent after a move. Two plan files that share a stem and both still
exist remain distinct plans. Prefix an explicit existing-graph dependency with `graph:` so it is not
remapped with internal plan dependencies.

### CLI subcommands

```text
godmode handon                                  # session-start triage summary
godmode handoff                                 # session-end validation
godmode context [--json]                        # full session context for hooks/agents
godmode status [--compact]                      # graph counts + next runnable tasks
godmode task list [--priority high|normal|low]
godmode task next [--priority high|normal|low]
godmode task add <title> [--id t5] [--depends-on t1,t2] [--crate-name X]
godmode task start <id>
godmode task done <id> [--commit <sha>] [--notes <text>]
godmode task block <id> <reason>
godmode task unblock <id>
godmode task unblock-all                        # reset all blocked tasks to pending
godmode task remove <id>
godmode task clear --done
godmode task clear --all
godmode task run <id> [--auto-done]             # execute task's run: field via rx
godmode task pull [--project <name>]            # import pending doob todos
godmode task pull --github [--repo owner/repo] [--label <label>]
godmode task apply <name> [--var k=v]           # expand a template into the task graph
godmode task list-templates                     # list available templates (local + global)
godmode task push-done                          # sync completed tasks back to doob
godmode plan ingest <path>                      # parse plan markdown into task graph
godmode dispatch [--max N] [--critical-path]    # emit parallel chains JSON
godmode visualize-graph [--format dot|svg] [--out <path>]  # render task graph
godmode pin [<path>]                            # pin session to a repo root
godmode unpin                                   # remove the pinned root
godmode init                                    # first-time setup: global config + state dirs
godmode doctor                                  # validate environment: tools, 1Password, worktrees
godmode memory-banking inject
godmode memory-banking remind
godmode memory-banking init
godmode memory-banking status
godmode agent list [--filter <kw>]              # list installed agents
godmode agent index                             # regenerate agents/INDEX.md
godmode agent dispatch <path> [--max N]         # plan ingest + dispatch in one shot
godmode agent generate [<name>] [--all]         # generate .md from agent YAML
godmode agent migrate [<name>] [--all]          # migrate agent .md frontmatter to YAML stubs
godmode graph build [--input <tmpl>] [--var k=v]
godmode verify [--crate-name X]                 # nextest + clippy + fmt + commits
godmode wave init --wave N --agents a,b,c
godmode wave status
godmode wave done <agent> [--commits <commits>]
godmode wave block <agent> <reason>
godmode wave check
godmode worktree add <branch> [--issue N]
godmode worktree remove <branch>
godmode ci triage [--run-id <id>]
godmode issue list [--repo owner/repo] [--label <label>]
godmode issue close <number> --commit <sha> [--repo owner/repo]
godmode skill list                              # list registered skills
godmode skill install <path>                    # install skill from a local dir
godmode skill uninstall <name>                  # remove a skill from the registry
godmode release current                         # show current plugin version
godmode release bump [--version X]              # increment patch version everywhere
godmode release tag                             # create annotated git tag
godmode release push                            # push branch + tag to origin
godmode release changelog                       # generate/prepend changelog entry
godmode release validate                        # cross-check plugin.json/Cargo.toml/tag
godmode pipeline list                           # list available pipelines
godmode pipeline show <name>                    # show steps + current position
godmode pipeline start <name> [--from <skill>]  # activate a pipeline
godmode pipeline next                           # mark current step done, advance
godmode pipeline skip                           # advance without marking done
godmode pipeline stop                           # deactivate current pipeline
godmode pipeline status                         # active pipeline + progress
godmode pipeline run <name> [--from <skill>] [--fail-fast]  # run headlessly
godmode policy resolve <agent> [--level L]      # effective policy for an agent
godmode policy check <agent> <tool> [--input <text>] [--level L]  # check a tool call
godmode policy list                             # list default/category/level policies
godmode policy audit [--date YYYY-MM-DD]        # governance audit trail
godmode hook list
godmode hook log [--tail N]
godmode hook test <script>
godmode hook migrate
godmode hook run <name>
godmode hook generate --client <claude|opencode>   # hooks.json / plugins/godmode.ts
godmode skill list
godmode skill install <path>
godmode skill uninstall <name>
godmode skill index [--check]
godmode review self
godmode review skills
godmode review agents
godmode release current
godmode release bump [--version X]
godmode release tag
godmode release push
godmode release changelog
godmode release validate
godmode trace tail [--n N] [--session ID|--current]
godmode trace failures [--session ID|--current]
godmode trace stats [--session ID|--current]
godmode trace summary [--sessions N] [--previous]
godmode hook generate --client <claude|opencode> [--dry-run]
godmode agent generate-opencode --all [--dry-run]
godmode insight add <title> --body <text> [--tags t1,t2]
godmode insight list [--date YYYY-MM-DD] [--json]
godmode insight render [--date YYYY-MM-DD]
godmode session prune --older-than <days> [--dry-run]
godmode policy resolve <agent> [--level <level>] [--json]
godmode policy check <agent> <tool> [--input <content>] [--level <level>]
godmode policy list [--json]
godmode policy audit [--date YYYY-MM-DD] [--json]
godmode workflow run <agent> <workflow>
godmode workflow list [--agent <name>]
godmode workflow status <name>
godmode scaffold <crate> <dimension>                    # test stub generator
godmode test-check <path>                               # check if .rs file has tests
```

`task done` accepts `--commit <sha>` and `--notes <text>` for trace metadata.

### Pipeline

```text
godmode pipeline list                           # show all pipelines
godmode pipeline show <name>                    # show steps with current position
godmode pipeline start <name> [--from <skill>]  # start and auto-invoke first step
godmode pipeline next                           # advance and invoke next step
godmode pipeline skip                           # advance without invoking
godmode pipeline stop                           # deactivate pipeline, preserve state
godmode pipeline status                         # show active pipeline + position
godmode pipeline run <name> [--from <skill>] [--fail-fast]  # headless: walk task graph, execute run: fields
```

Six pipelines are defined in `pipelines/`:

- `feature` — idea to merged PR
- `parallel-feature` — fan-out across crates
- `release` — health check, audit, changelog, tag
- `maintenance` — health scorecard + targeted cleanup
- `triage` — issue backlog to task graph
- `retrospective` — session reflection and learning

Pipeline state persists in `.ctx/godmode/pipeline.yaml` (gitignored).
`task clear` requires `--done` (completed only) or `--all`.

## Plugin layout

This repo is also a Claude Code plugin installed via bazaar:

```text
.claude-plugin/plugin.json   # name, version, author, description only — no extra fields
skills/                      # discovered by directory scan, not declared in plugin.json
agents/                      # top-level *.md are GENERATED — Claude discovers these
  cfg/*.cfg.yaml             # source of truth: structured agent config
  prompts/*.prompt.txt       # source of truth: raw prompt text
  INDEX.md                   # generated: agent table
```

Plugin manifest schema accepts only: `name`, `version`, `author`, `description`. Extra fields
cause validation failure on `claude plugin install`.

## OpenCode parity

`.opencode/` holds the OpenCode projection. Everything in it is GENERATED — edit the
`agents/cfg/*.cfg.yaml` sources or the hook registry, then regenerate.

| Surface  | Source of truth                                           | Generated output               | Regenerate with                           |
| -------- | --------------------------------------------------------- | ------------------------------ | ----------------------------------------- |
| Commands | `commands/gm/*.yaml`                                      | `.opencode/commands/gm-*.md`   | `nu commands/gm/generate.nu`              |
| Agents   | `agents/cfg/*.cfg.yaml` + `agents/prompts/*.txt`          | `.opencode/agents/*.md`        | `godmode agent generate-opencode --all`   |
| Hooks    | `REGISTRY` in `crates/godmode-core/src/hooks/registry.rs` | `.opencode/plugins/godmode.ts` | `godmode hook generate --client opencode` |
| Skills   | `skills/*/SKILL.md`                                       | none needed                    | —                                         |

Skills need no projection: OpenCode reads `.claude/skills/*/SKILL.md` natively.

Agent projections are deny-by-default. Granted tools become `permission: allow`, followed by a
`"*": deny` catch-all, because OpenCode applies the _last_ matching rule — the catch-all must
come last or it would be overridden.

Hooks map Claude's `PreToolUse`/`PostToolUse` to OpenCode's `tool.execute.before`/`after`.
Claude's `SessionStart` and `Stop` have **no OpenCode equivalent**, so these 6 hooks do not run
under OpenCode and are reported by a warning at plugin load:

`session-start`, `memory-bank-inject`, `task-management` (SessionStart) — `stop-guard`,
`memory-bank-update-remind`, `introspection` (Stop)

`UNSUPPORTED_OPENCODE_EVENTS` in `registry.rs` is the single source of truth for that list;
`opencode_unmapped()` derives the report from it. Do not hand-edit
`.opencode/plugins/godmode.ts` — three conformance tests fail if it drifts.

### OpenCode gates

`taskit check ci` only dispatches Rust built-ins, so the OpenCode projection is gated in
GitHub Actions (`conformance-opencode` job) rather than taskit. Run both locally before
committing a change to `.opencode/` or `crates/godmode-core/src/hooks/registry.rs`:

```bash
bun install --cwd .opencode                # audit reads the installed type definitions
bun run scripts/opencode/audit-plugin.ts   # contract audit
bun x tsc -p .opencode/tsconfig.json       # typecheck
```

`audit-plugin.ts` derives the hook contract by parsing the installed
`@opencode-ai/plugin/dist/index.d.ts` rather than hardcoding hook names, so a hook OpenCode
never declared is caught. It is not redundant with `tsc`: TypeScript does **not** enforce
handler arity, and does **not** flag an unknown property when a valid key is also present.
A 1-arg handler satisfying a 2-arity hook, or a typo'd hook name beside a real one, both
compile clean.

`.opencode/package.json` and `bun.lock` are tracked on purpose. The audit reads the pinned
`@opencode-ai/plugin` version, so leaving them gitignored would make the gate
unreproducible and unable to run in CI.

## Gotchas

- `ls <dir> | wc -l` in the Nushell tool wrapper counts **table rows, not entries**. Use
  `rg --files <dir> | wc -l` or a Python one-liner when counting files.
- Bash heredocs (`cat << 'EOF'`) do not survive the Nushell tool wrapper — write to a file with
  the Write tool, then `cat <file> >> target`. Same for `cmd 2>&1`, `cmd && cmd`, and `||`.
- CLI Quick Reference (`## CLI subcommands`) can silently drift from `crates/godmode-cli/src/main.rs`
  — when adding/changing a `Cmd` variant, grep `enum.*Action` in `main.rs` and diff against the
  reference block. `skill`/`release`/`pipeline`/`policy` families were undocumented for a while.
- `skills/introspection/helpers/audit.nu` checks skill-index completeness and cross-references
  subcommand calls; run it after editing any `skills/*/SKILL.md` or `agents/*.md`. Report lands in
  `.ctx/godmode/reports/introspection/` (gitignored).
- `godmode plan ingest` safely coexists with earlier plans. Colliding IDs are namespaced by the plan
  filename, internal dependencies are rewritten, and JSON output reports actual added/skipped IDs.
- `godmode task add <title> --id <id> --depends-on ""` registers an empty string as a dep,
  causing "unmet dependencies" on start. Omit `--depends-on` entirely for root tasks.
- `dispatch --critical-path` shows the critical path tasks; `godmode status` also surfaces it.
- Pre-commit hook runs `taskit check fmt --check` and fails the commit on drift — it checks but
  does not fix. A PostToolUse hook runs fmt on edited `.rs` files but does NOT auto-stage; run
  `taskit check fmt` then `git add` the reformatted files, or the pre-commit check still fails.
- `plan::parse` returns `Result<Vec<Task>>`, not `Vec<Task>` — always match/unwrap the Result.
- `dispatch::independent_chains(graph, max)` returns `Vec<Chain>` — not `build_slots`.
- `tests/conformance/` is a workspace member (`-p godmode-conformance`); add new test modules
  in `src/`, register in `lib.rs::all_tests()`, and add `pub mod` to `lib.rs`.
- `Task::started_at` is set by `Session::start_task`, not `graph::start` — duration tracking
  only works when transitions go through `Session`, not raw `graph::*` functions directly.
- `rx::validate_run` fires inside `Session::start_task` before state mutation — if the script
  doesn't exist and `rx` is on PATH, the task is rejected before being marked Running.

## Rust Conventions

- Gates go through `taskit` — never invoke `cargo` directly.
- Do NOT run gates by hand before committing or pushing. The global git hooks run format
  check, clippy on affected crates, tests, and the secret scan. Fix what a hook reports.
- A format failure is the one case the hook detects but does not fix: run `taskit check fmt`
  then `git add` the reformatted files.
- Clippy runs with `-D warnings`; fix warnings proactively rather than suppressing them.
- Do not investigate rust-analyzer or IDE diagnostics unless explicitly asked — they are often
  stale.

## CI

Inspect recent runs on the current branch without opening a TTY watcher:

```bash
gh run list --branch "$(git branch --show-current)" --limit 3
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
- Worktree subagents commit and report their branch/SHA without integrating or removing the
  worktree. The orchestrator delegates integration and cleanup to wave integration.
- After subagents complete, verify their changes were committed (`git log --oneline -3`).
  A HANDOFF with `commits: []` is incomplete.
- Merge parallel branches sequentially with `git merge --no-ff`; never cherry-pick or use an
  octopus merge.
- Cap parallel subagents at 5 concurrent to avoid API rate limits.
- Never use `--no-verify` in subagent git operations.
- After 3 failed attempts, write `BLOCKED.md` and stop before escalating.

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
