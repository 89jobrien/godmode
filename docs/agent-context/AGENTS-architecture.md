# Architecture

Read this before changing code in either crate or locating where logic lives.

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
