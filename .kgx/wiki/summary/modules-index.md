# Modules Index

All 27 public modules in `godmode-core/src/`:

## Domain Core

| Module          | Responsibility                                                              |
| --------------- | --------------------------------------------------------------------------- |
| `model`         | Task, TaskGraph, Status, Priority — the data model                          |
| `graph`         | Load/save tasks.yaml, state transitions, `runnable()` dependency resolution |
| `session`       | Session struct — owns transitions, duration tracking, cruxx traces          |
| `session_trace` | Low-level JSONL append helpers for session                                  |
| `plan`          | Parse plan markdown (`### Task N:`) into Task structs                       |
| `dispatch`      | Independent chains for parallel agent execution (max 5)                     |
| `wave`          | Parallel agent slot state — init, done, blocked, check                      |
| `builder`       | Interactive + file-driven graph construction (shape/wire/validate phases)   |

## Infrastructure

| Module    | Responsibility                                                    |
| --------- | ----------------------------------------------------------------- |
| `detect`  | Walk up from CWD to git root, read Cargo.toml package name        |
| `config`  | Load `.godmode.toml` or `~/.config/godmode/config.toml`           |
| `context` | SessionContext for hooks and subagents                            |
| `cache`   | StatusCache to `~/.cache/godmode/status.json` for starship prompt |
| `verify`  | nextest + clippy + fmt + git log gate                             |
| `init`    | Project initialization                                            |
| `doctor`  | Workspace health checks                                           |

## Agent & Plugin

| Module        | Responsibility                                                 |
| ------------- | -------------------------------------------------------------- |
| `agent`       | AgentDef from `agents/cfg/*.cfg.yaml` + `prompts/*.prompt.txt` |
| `agent_index` | Regenerate `agents/INDEX.md`                                   |
| `skill`       | Skill registry install/uninstall                               |
| `registry`    | RegistryEntry persistence at `~/.config/godmode/registry.json` |
| `review`      | Plugin conformance auditing                                    |
| `release`     | Version bump, tag, push, changelog                             |

## Integrations (subprocess wrappers)

| Module  | Wraps     | Purpose                       |
| ------- | --------- | ----------------------------- |
| `cruxx` | cruxx CLI | Step trace writes             |
| `doob`  | doob CLI  | Todo sync, handoff sync       |
| `hj`    | hj CLI    | Handoff YAML read/write       |
| `rx`    | rx CLI    | Script dispatch, validate_run |
| `gh`    | gh CLI    | CI triage, GitHub issues      |

## Supporting

| Module           | Responsibility                                                 |
| ---------------- | -------------------------------------------------------------- |
| `templates`      | `{{var}}` substitution, apply to graph                         |
| `insights`       | Append-only JSONL insight capture                              |
| `workflow`       | Causal workflow DAGs per agent                                 |
| `worktree`       | Git worktree lifecycle                                         |
| `scaffold`       | Test stub generator                                            |
| `test_check`     | Check if .rs files have tests                                  |
| `pipeline`       | Pipeline execution                                             |
| `report_index`   | SOLID report indexing                                          |
| `memory_banking` | `.ctx/memory-bank/` context generation                         |
| `hooks`          | auto_block, hook_context, pre_commit, quality_gate, stop_guard |
