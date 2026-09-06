# Plan: Daily Orchestration Health Remediation

**Status**: active — cross-repository remediation remains in progress

## Goal

Make the daily sweep report command failures truthfully, avoid self-inflicted resource contention, and clear or deliberately contain every confirmed failure and warning from the 2026-08-26 run.

## Contents

- Context Map
- Finding Ledger
- Architecture and Tech Stack
- Worktree Setup
- Tasks
- Deferred Operator Actions
- Verification Matrix

## Context Map

### Files to Modify

| File                                                                                       | Purpose                          | Changes Needed                                                                                                                 |
| ------------------------------------------------------------------------------------------ | -------------------------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| `/Users/joe/.notfiles/claude/.claude/skills/daily-orchestration/SKILL.md`                  | Canonical orchestration skill    | Pull from configured upstream, sequence Cargo checks, preserve non-zero exits, add Pieces readiness canary and circuit breaker |
| `/Users/joe/.notfiles/claude/.claude/skills/daily-orchestration/tests/failing-health.crux` | New orchestration fixture        | Prove a tolerated lint failure does not stop the later test step                                                               |
| `/Users/joe/.notfiles/claude/.claude/skills/daily-orchestration/tests/validate.nu`         | New orchestration validator      | Validate fixture trace exit codes and required skill policy text                                                               |
| `/Users/joe/.notfiles/claude/.claude/skills/pieces-health/SKILL.md`                        | New canonical Pieces diagnostics | Adopt the active standalone skill, separate liveness from readiness, and bound retries                                         |
| `/Users/joe/.notfiles/claude/.claude/skills/pieces-ltm/SKILL.md`                           | LTM guidance                     | Use the readiness canary and degraded mode                                                                                     |
| `/Users/joe/.notfiles/claude/.claude/skills/pieces/SKILL.md`                               | Pieces overview                  | Correct health endpoint semantics                                                                                              |
| `/Users/joe/dev/devloop/crates/devloop/src/adapters/baml/council_analyzer.rs`              | Partial synthesis adapter        | Remove redundant formatting borrow                                                                                             |
| `/Users/joe/dev/devloop/.config/nextest.toml`                                              | Nextest policy                   | Remove stale override for deleted test                                                                                         |
| `/Users/joe/dev/devkit/internal/ai/providers/router_test.go`                               | Provider smoke tests             | Require explicit live-test opt-in                                                                                              |
| `/Users/joe/dev/devkit/internal/ai/meta/exec.go`                                           | New meta orchestration API       | Restore the `meta.Exec` API already consumed by both CLIs                                                                      |
| `/Users/joe/dev/devkit/internal/ai/meta/exec_test.go`                                      | New meta orchestration tests     | Cover output, logs, no-synthesis output, and runner errors                                                                     |
| `/Users/joe/dev/taskit/crates/taskit-init/src/scaffold.rs`                                 | xtask templates                  | Generate rustfmt-clean source and test it                                                                                      |
| `/Users/joe/dev/notfiles/xtask/src/main.rs`                                                | Generated task runner            | Apply canonical rustfmt output only                                                                                            |
| `/Users/joe/dev/romp/src/main.rs`                                                          | Romp binary                      | Apply rustfmt output                                                                                                           |
| `/Users/joe/dev/romp/src/proxy/mod.rs`                                                     | Proxy exports                    | Apply rustfmt ordering                                                                                                         |
| `/Users/joe/dev/romp/src/proxy/webhook.rs`                                                 | Webhook types                    | Apply rustfmt output                                                                                                           |
| `/Users/joe/dev/romp/src/proxy/db_loggable.rs`                                             | Proxy log tests                  | Initialize headers without field reassignment                                                                                  |
| `/Users/joe/dev/gooey/baml_client/src/functions.rs`                                        | Macro-generated functions        | Contain macro-generated missing-doc warnings at module scope                                                                   |
| `/Users/joe/dev/gooey/baml_client/src/providers.rs`                                        | Macro-generated providers        | Contain macro-generated missing-doc warnings at module scope                                                                   |
| `/Users/joe/dev/gooey/baml_client/examples/productivity_insights.rs`                       | Example analyzer                 | Remove dead duplicate fields                                                                                                   |
| `/Users/joe/dev/gooey/baml_client/examples/comprehensive_metrics.rs`                       | Example analyzer                 | Remove dead duplicate fields                                                                                                   |
| `/Users/joe/dev/gooey/baml_client/examples/weekly_trends.rs`                               | Example analyzer                 | Remove dead method                                                                                                             |
| `/Users/joe/dev/gooey/baml_client/examples/baml_transcript_analyzer.rs`                    | Example analyzer                 | Remove unused import and print session ID                                                                                      |
| `/Users/joe/dev/gooey/baml_client/examples/analyze_hook.rs`                                | Example analyzer                 | Remove unused type import                                                                                                      |
| `/Users/joe/dev/gooey/baml_client/examples/pattern_analysis.rs`                            | Example analyzer                 | Store timestamps instead of dead event fields                                                                                  |
| `/Users/joe/dev/gooey/baml_client/examples/tool_patterns.rs`                               | Example analyzer                 | Retain only consumed tool sequence data                                                                                        |
| `/Users/joe/dev/steve/scripts/fix_frontmatter.py`                                          | Frontmatter repair script        | Repair invalid multiline regex                                                                                                 |
| `/Users/joe/dev/steve/scripts/fix_frontmatter_strict.py`                                   | Strict repair script             | Repair invalid multiline regex and split one-line statements                                                                   |
| `/Users/joe/dev/steve/scripts/tests/test_fix_frontmatter.py`                               | New script regression tests      | Cover extraction and filesystem isolation                                                                                      |
| `/Users/joe/dev/steve/Makefile`                                                            | Python quality commands          | Separate blocking syntax/security lint from full debt inventory                                                                |
| `/Users/joe/dev/steve/steve/skills/jira/scripts/jira-api.py`                               | Jira helper                      | Validate HTTPS Jira origins before authenticated URL access                                                                    |
| `/Users/joe/dev/steve/steve/skills/testing/scripts/with-server.py`                         | Local test server helper         | Document trusted shell boundary for intentional shell syntax                                                                   |
| `/Users/joe/dev/inflection/mise.toml`                                                      | Toolchain                        | Pin golangci-lint 2.13.1                                                                                                       |
| `/Users/joe/dev/inflection/Makefile`                                                       | Go quality commands              | Run the mise-pinned linter and require it in tool checks                                                                       |
| `/Users/joe/dev/inflection/README.md`                                                      | Developer setup                  | Document canonical lint setup                                                                                                  |
| `/Users/joe/dev/inflection/AGENTS.md`                                                      | Agent setup                      | Document canonical lint setup                                                                                                  |
| `/Users/joe/dev/inflection/internal/secrets/import_csv_test.go`                            | Secrets tests                    | Replace process-global environment mutation with `t.Setenv`                                                                    |
| `/Users/joe/dev/inflection/internal/secrets/store_test.go`                                 | Secrets tests                    | Replace process-global environment mutation with `t.Setenv`                                                                    |
| `/Users/joe/dev/inflection/internal/indexer/indexer.go`                                    | SQLite indexer                   | Handle resource-close results consistently                                                                                     |
| `/Users/joe/dev/inflection/internal/sink/file_sink.go`                                     | Event sink                       | Handle file-close results correctly                                                                                            |
| `/Users/joe/dev/inflection/internal/viewer/viewer.go`                                      | TUI model                        | Preserve search-count state and apply staticcheck fixes                                                                        |
| `/Users/joe/dev/inflection/internal/viewer/viewer_test.go`                                 | TUI tests                        | Cover search-count updates                                                                                                     |
| `/Users/joe/dev/inflection/cmd/inflection/version.go`                                      | Build version                    | Expose linker-injected version through CLI                                                                                     |
| `/Users/joe/dev/inflection/cmd/inflection/main.go`                                         | CLI flags                        | Handle version-only execution                                                                                                  |
| `/Users/joe/dev/inflection/cmd/inflection/main_test.go`                                    | CLI tests                        | Cover version output without starting TUI                                                                                      |
| `/Users/joe/dev/magi/pyproject.toml`                                                       | Python dependencies              | Require SentencePiece 0.2.2 or newer within 0.2                                                                                |
| `/Users/joe/dev/magi/uv.lock`                                                              | Locked dependencies              | Resolve SentencePiece 0.2.2                                                                                                    |
| `/Users/joe/dev/magi/tests/test_deps_smoke.py`                                             | Dependency smoke tests           | Assert SentencePiece import and local processor construction                                                                   |
| `/Users/joe/dev/doob/crates/doob/tests/beads_adapter_test.rs`                              | Stale integration test           | Remove obsolete cfg-gated test crate                                                                                           |
| `/Users/joe/dev/doob/crates/doob/tests/beads_integration_test.rs`                          | Stale integration test           | Remove obsolete cfg-gated test crate                                                                                           |
| `/Users/joe/dev/doob/crates/doob/tests/common/mod.rs`                                      | Shared test helpers              | Stop importing sync mocks into every integration-test crate                                                                    |
| `/Users/joe/dev/doob/crates/doob/tests/common/sync_mocks.rs`                               | Sync fixtures                    | Remove genuinely unused fixture and imports                                                                                    |
| `/Users/joe/dev/doob/crates/doob/tests/sync_domain_test.rs`                                | Sync tests                       | Import sync fixtures directly                                                                                                  |
| `/Users/joe/dev/doob/crates/doob/tests/sync_service_test.rs`                               | Sync tests                       | Import sync fixtures directly                                                                                                  |
| `/Users/joe/dev/doob/crates/doob-sync/src/service.rs`                                      | Sync service tests               | Replace boolean equality assertion                                                                                             |
| `/Users/joe/dev/doob/crates/doobdash/src/ui.rs`                                            | TUI formatting                   | Remove redundant formatting borrow                                                                                             |
| `/Users/joe/dev/doob/crates/doob/src/commands/update.rs`                                   | Update tests                     | Use snake-case test names                                                                                                      |
| `/Users/joe/dev/doob/crates/doob/tests/context_integration_test.rs`                        | Context integration              | Serialize current-directory tests unconditionally                                                                              |
| `/Users/joe/dev/doob/crates/doob/tests/context_test.rs`                                    | Context unit integration         | Stop importing unused shared helpers                                                                                           |
| `/Users/joe/dev/personal-mcp/src/proc_gc.rs`                                               | Process cleanup tool             | Fail closed before live signaling and test denial paths                                                                        |
| `/Users/joe/dev/personal-mcp/src/main.rs`                                                  | MCP tool registration            | State mutation permission requirement                                                                                          |
| `/Users/joe/dev/tools/.gitignore`                                                          | Repository hygiene               | Ignore target and macOS metadata                                                                                               |
| `/Users/joe/dev/tools/AGENTS.md`                                                           | New repository guidance          | Document actual single-crate commands and artifact policy                                                                      |
| `/Users/joe/dev/dumcp/.gitignore`                                                          | Repository hygiene               | Keep `.envrc` excluded                                                                                                         |
| `/Users/joe/dev/dumcp/README.md`                                                           | Project documentation            | Commit the reviewed code-grounded README                                                                                       |
| `/Users/joe/dev/dumcp/AGENTS.md`                                                           | New repository guidance          | Document Go and filesystem-jail invariants                                                                                     |
| `/Users/joe/dev/dumcp/main_test.go`                                                        | Existing Go tests                | Apply behavior-neutral gofmt alignment                                                                                         |
| `/Users/joe/dev/pieces-ob/.env.example`                                                    | Environment guidance             | Keep corrected Pieces port wording                                                                                             |
| `/Users/joe/dev/pieces-ob/.gitignore`                                                      | Repository hygiene               | Ignore `.DS_Store` and local `.ctx` state                                                                                      |
| `/Users/joe/dev/pieces-ob/AGENTS.md`                                                       | New repository guidance          | Document uv, PiecesOS, Obsidian, and credential boundaries                                                                     |
| `/Users/joe/dev/goder/.gitignore`                                                          | Repository hygiene               | Keep `.envrc` excluded                                                                                                         |

### Dependencies

| Source                           | Relationship                       | Consumer                                                      |
| -------------------------------- | ---------------------------------- | ------------------------------------------------------------- |
| Canonical daily skill            | Symlink                            | `/Users/joe/.claude/skills/daily-orchestration/SKILL.md`      |
| Generated Crux YAML              | Schema and runner                  | `crux-script` and `crux-stdlib::shell::capture`               |
| Pieces liveness/readiness policy | Controls fan-out                   | All per-repository analysis agents                            |
| taskit xtask templates           | Generate source                    | notfiles `xtask/src/main.rs` and future taskit consumers      |
| Devkit environment gate          | Controls live calls                | `TestRouterSmoke` only; production router wiring is unchanged |
| Gooey macros                     | Generate public APIs               | `baml-client` missing-doc lint                                |
| Doob common test module          | Compiled once per integration test | All files under `crates/doob/tests/`                          |
| personal-mcp mutation gate       | Authorizes signaling               | `proc_gc` live execution path                                 |

### Test Coverage

| Test or Gate                                          | Covers                                              |
| ----------------------------------------------------- | --------------------------------------------------- |
| `crux check` plus a generated failing-command fixture | Non-zero health steps remain visible and tolerated  |
| `personal_agentlint_check` on canonical skills        | Skill structure and references                      |
| Devloop package Clippy and nextest                    | Redundant borrow and nextest configuration          |
| `TestRouterSmoke` targeted Go test                    | Default skip and explicit live opt-in               |
| `taskit-init` scaffold tests                          | Fresh and injected xtask generation                 |
| Romp fmt, Clippy, and nextest                         | Formatting-only remediation                         |
| Gooey `baml-client` Clippy                            | Macro-generated missing-doc containment             |
| New Steve tmp-path tests                              | Regex extraction without rewriting repository files |
| Inflection lint, vet, and tests                       | Pinned linter plus normal Go gates                  |
| Magi dependency smoke test                            | SentencePiece import without model download         |
| Doob all-target/all-feature Clippy and tests          | Integration-test warning cleanup                    |
| personal-mcp focused and full tests                   | Mutation denial without real `kill(2)`              |

### Reference Patterns

| File                                                            | Pattern                          |
| --------------------------------------------------------------- | -------------------------------- |
| `/Users/joe/dev/crux/crates/crux-script/tests/allow_failure.rs` | Tolerated top-level failures     |
| `/Users/joe/dev/crux/crates/crux-script/tests/timeout_ms.rs`    | Per-step timeout                 |
| `/Users/joe/dev/devkit/internal/ai/baml/integration_test.go`    | Explicit integration gating      |
| `/Users/joe/dev/personal-mcp/src/project_tools/minibox.rs`      | `require_project_mutations` gate |
| `/Users/joe/dev/steve/scripts/tests/test_add_metadata.py`       | Isolated script tests            |
| `/Users/joe/dev/magi/tests/test_deps_smoke.py`                  | Dependency smoke-test style      |

### Risk

- [ ] Never edit the symlinked daily skill through the stale Maestro workspace copy.
- [ ] Do not run Cargo lint and tests concurrently in one workspace target directory.
- [ ] Do not retry Pieces LTM per repository after the run-wide readiness canary fails.
- [ ] Do not stage existing dirty work with `git add -A`; every commit uses explicit paths.
- [ ] Do not run Steve frontmatter scripts against repository content during tests.
- [ ] Do not edit Cargo registry macro sources; contain locally or release an upstream fix.
- [ ] Do not call real `kill(2)` from personal-mcp tests.
- [ ] Do not enable stale Doob cfg features; the tests reference removed APIs.
- [ ] Do not delete Kan, Doob, or handoff artifacts without separate operator approval.
- [ ] Do not treat Minibox as broken: commit `3f226924` already fixed the traced findings.

## Finding Ledger

| ID  | Confirmed finding                                                          | Resolution                                     |
| --- | -------------------------------------------------------------------------- | ---------------------------------------------- |
| F01 | Crux `ignore_exit` produced false-green health steps                       | Task 2                                         |
| F02 | Cargo lint/test fan-out caused lock contention and timeouts                | Task 2                                         |
| F03 | PiecesOS liveness passed while LTM was saturated and unavailable           | Tasks 3 and 4                                  |
| F04 | Braid's Gitea host is offline while GitHub is healthy                      | Task 1 plus deferred operator action           |
| F05 | Doob has no upstream for its local branch                                  | Task 1 plus deferred operator action           |
| F06 | Devloop has one Clippy error and one stale nextest override                | Tasks 5 and 6                                  |
| F07 | Minibox trace predates an already-applied fix                              | Already-resolved verification section          |
| F08 | Devkit ordinary tests can make live provider calls                         | Task 7                                         |
| F09 | Taskit emits unformatted xtask source and Notfiles consumes it             | Tasks 8 through 10                             |
| F10 | Romp formatting fails and cold concurrent compilation exceeded 120 seconds | Tasks 2 and 11                                 |
| F11 | Gooey macros emit six undocumented public APIs                             | Task 12                                        |
| F12 | Steve has two syntax-broken scripts and 514 full-scope Ruff findings       | Tasks 13 through 15 plus deferred debt program |
| F13 | Inflection requires an absent, unpinned linter                             | Task 16                                        |
| F14 | Magi SentencePiece 0.2.1 emits Python 3.13 SWIG warnings                   | Task 17                                        |
| F15 | Doob stale cfg tests and shared fixtures produce compiler warnings         | Tasks 18 and 19                                |
| F16 | personal-mcp live process cleanup lacks mutation gating                    | Task 20                                        |
| F17 | Tools tracks build artifacts and lacks accurate local guidance             | Task 21                                        |
| W01 | Dumcp, Pieces Obsidian, and Goder contain small stale hygiene changes      | Tasks 22 through 24                            |
| W02 | Kan mixes feature work with generated artifacts                            | Deferred operator action                       |
| W03 | Obfsck handoff state and provider comment are unrelated                    | Deferred operator action                       |
| F18 | Devkit consumers reference a missing `meta.Exec` API                       | Task 25                                        |
| F19 | Steve blocking lint exposes three security-boundary findings               | Task 26                                        |
| F20 | Inflection's pinned linter exposes 58 uncapped findings                    | Tasks 27 through 31                            |
| F21 | Doob pre-commit is blocked by two baseline Clippy findings                 | Task 32                                        |
| F22 | Dumcp documentation commit is blocked by baseline gofmt                    | Task 33                                        |
| F23 | Romp all-target Clippy exposes field reassignment in a test                | Task 34                                        |
| W04 | Doob nextest emits naming, cfg, and unused-helper warnings                 | Task 35                                        |
| W05 | Gooey example targets emit dead-code and unused-import warnings            | Task 36                                        |

## Architecture

- Crates and projects affected: daily-orchestration skills, Pieces guidance, devloop, devkit, taskit-init, notfiles, romp, gooey/baml-client, Steve, inflection, magi, doob, personal-mcp, tools, dumcp, pieces-ob, and goder.
- New traits/types: none. Steve adds only test helpers; personal-mcp may add a private signal callback to isolate `kill(2)`.
- Data flow: repository registry -> upstream-safe pull -> one Pieces readiness canary -> bounded LTM context -> sequential Crux status/lint/test steps -> trace exit classification -> P0 dispatch -> daily note -> upstream-safe push.

## Tech Stack

- Rust 2024/2021 as already selected by each workspace; Tokio, Clippy, nextest, rustfmt, Crux DSL.
- Go 1.25.5 with golangci-lint 2.13.1.
- Python 3.11+ with uv, Ruff, pytest, and SentencePiece 0.2.2.
- No new runtime dependency is required.

## Worktree Setup

Create `/Users/joe/dev/.worktrees/daily-orchestration-2026-08-26` once. Each row defines the exact source repository, immutable base ref, and task worktree. For each row, pass its literal Source, Worktree, and Base cells to `git -C`, `git worktree add -b fix/daily-orchestration-health-2026-08-26`, and the final base-ref argument. If the branch already exists, stop and inspect it instead of reusing it blindly.

| Project         | Source                        | Base commit                                | Worktree                                                                   |
| --------------- | ----------------------------- | ------------------------------------------ | -------------------------------------------------------------------------- |
| notfiles-config | `/Users/joe/.notfiles`        | `06623d76daab086ff902bbed0627bd1b62d69825` | `/Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/notfiles-config` |
| devloop         | `/Users/joe/dev/devloop`      | `196fd65d3c76d61f52ab033b9bedc127b3ad475d` | `/Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/devloop`         |
| devkit          | `/Users/joe/dev/devkit`       | `02db45f14f0f893f62ac27e52aef0e9fce11de7d` | `/Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/devkit`          |
| taskit          | `/Users/joe/dev/taskit`       | `e4b371e4b34f66a1817b33b781f8ad36a966e7c9` | `/Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/taskit`          |
| notfiles        | `/Users/joe/dev/notfiles`     | `f93cc6ebeb1527cf316394fa120f597746ee305c` | `/Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/notfiles`        |
| romp            | `/Users/joe/dev/romp`         | `2816dc4d4e353ad77fd3e2b20414b9799bcefc26` | `/Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/romp`            |
| gooey           | `/Users/joe/dev/gooey`        | `ba1cb6226d51fc5d1fe40ead3ea17fdf910660bb` | `/Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/gooey`           |
| steve           | `/Users/joe/dev/steve`        | `9cea4fd4809ce8ada8a64edde534e88855e31807` | `/Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/steve`           |
| inflection      | `/Users/joe/dev/inflection`   | `6cc45cc789c5a96f48e6cc272ffca472ab650029` | `/Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/inflection`      |
| magi            | `/Users/joe/dev/magi`         | `bd66b31ea3ac8032d18e28454c4dcbcb97322f83` | `/Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/magi`            |
| doob            | `/Users/joe/dev/doob`         | `d06ea7ecc45eb05c1171e89269ed42520ca5018d` | `/Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/doob`            |
| personal-mcp    | `/Users/joe/dev/personal-mcp` | `8f211de56e4b8ffd8b156991697d6e359f3d7352` | `/Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/personal-mcp`    |
| tools           | `/Users/joe/dev/tools`        | `0a26ebd74ad4b020df52f7ebd700fe326f114eee` | `/Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/tools`           |
| dumcp           | `/Users/joe/dev/dumcp`        | `50ce6df4cf97817f7bf1e7524d1c3adb6a2d3768` | `/Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/dumcp`           |
| pieces-ob       | `/Users/joe/dev/pieces-ob`    | `a506fcae4cdc83b71f0702a630a4a912a42013be` | `/Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/pieces-ob`       |
| goder           | `/Users/joe/dev/goder`        | `dbc0b0c4a1789340064a2e4eab620b1950ebeb52` | `/Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/goder`           |
| minibox         | `/Users/joe/dev/minibox`      | `045070e8926941810fbe1c48663b9ea3640cffd0` | `/Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/minibox`         |

All task paths identify source files from the context map. Perform edits and commands at the corresponding exact worktree path above. Tasks 10, 20, 22, 23, and 24 contain their exact selected-change transfer steps. Never copy a whole dirty tree.

Before every commit, run `git branch --show-current`; require `fix/daily-orchestration-health-2026-08-26`. Stage only task paths.

## Tasks

### Task 1: Make pull and push upstream-aware

**Crate**: `dotfiles`
**File(s)**: `/Users/joe/.notfiles/claude/.claude/skills/daily-orchestration/SKILL.md`
**Run**: `nu -c 'open /Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/notfiles-config/claude/.claude/skills/daily-orchestration/SKILL.md | str contains "git pull --ff-only"'`

1. Confirm the Phase 1 pull example hard-codes `origin` and the current branch; expected result: true.
2. Replace Phase 1 with Nushell logic that obtains `@{upstream}`, skips with `NO_UPSTREAM`, and runs `git pull --ff-only` without naming a remote or branch. Apply the same upstream check before Phase 5 push.
3. Verify the Braid example resolves `github/main`, while Doob reports `NO_UPSTREAM` without attempting `origin feat/todo-add-due-flag`.
4. Commit only the canonical skill: `fix(orchestration): use configured repository upstreams`.

### Task 2: Preserve failing health exit codes and serialize Cargo gates

**Crate**: `dotfiles`
**File(s)**: `/Users/joe/.notfiles/claude/.claude/skills/daily-orchestration/SKILL.md`, `/Users/joe/.notfiles/claude/.claude/skills/daily-orchestration/tests/failing-health.crux`, `/Users/joe/.notfiles/claude/.claude/skills/daily-orchestration/tests/validate.nu`
**Run**: `nu -c 'cd /Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/notfiles-config; nu claude/.claude/skills/daily-orchestration/tests/validate.nu'`

1. Run the command before editing; expected result: exit 1.
2. First add `tests/failing-health.crux` with sequential `git_status`, `lint`, and `test` steps. `git_status` uses `git::status`; lint uses `shell::capture` with `cmd: "nu -c 'exit 7'"`, `allow_failure: true`, and `timeout_ms: 5000`; test uses `shell::capture` with `cmd: "nu -c 'exit 0'"` and `timeout_ms: 5000`.
3. Add `tests/validate.nu`. From the skill directory it must run these exact commands: `crux check tests/failing-health.crux`, `crux run --dry-run --strict tests/failing-health.crux`, and `crux run --strict --save-trace /tmp/daily-orchestration-fixture.json tests/failing-health.crux`. Parse the trace and assert the lint step is not `ok`, the test step is `ok`, and lint precedes test by `started_at`.
4. Run the validator before editing the skill; expected result: fail because the skill still prescribes `ignore_exit` and `join_all`.
5. Replace the generated pipeline with three sequential top-level steps. Use this exact shape for lint and test:

   ```yaml

   ```

- step: lint
  handler: shell::capture
  allow_failure: true
  timeout_ms: 120000
  args:
  cmd: "cargo clippy --all-targets -- -D warnings"

````
Generate this exact Rust command, `go vet ./...`, `uv run ruff check .`, or `git status --short` according to the existing language map; project-native commands from AGENTS.md replace only their language default before the file is written.
6. Remove `ignore_exit`; require agents to classify the failed-allowed trace as a warning/error.
7. Run `tests/validate.nu`; all three fixture steps must execute and the validator must exit 0.
8. Commit the skill and both fixture files: `fix(orchestration): retain health command failures`.

### Task 3: Add one Pieces readiness canary and a run-wide circuit breaker

**Crate**: `dotfiles`
**File(s)**: `/Users/joe/.notfiles/claude/.claude/skills/daily-orchestration/SKILL.md`
**Run**: `nu -c 'open /Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/notfiles-config/claude/.claude/skills/daily-orchestration/SKILL.md | str contains "PIECES_LTM_READY"'`

1. Run before editing; expected result: false.
2. After HTTP liveness succeeds, perform one 15-second `ask_pieces_ltm` canary with the prompt `Return only READY.`.
3. Set `PIECES_LTM_READY=true` only on a useful response. On timeout/error, set it false for the whole run and issue no per-repository LTM calls.
4. When ready, cap LTM calls at two concurrent requests independently of the five-agent analysis cap; after two later failures, disable remaining LTM calls.
5. Commit: `fix(orchestration): gate LTM fanout on readiness`.

### Task 4: Align Pieces health and LTM guidance

**Crate**: `dotfiles`
**File(s)**: `/Users/joe/.notfiles/claude/.claude/skills/pieces-health/SKILL.md`, `/Users/joe/.notfiles/claude/.claude/skills/pieces-ltm/SKILL.md`, `/Users/joe/.notfiles/claude/.claude/skills/pieces/SKILL.md`
**Run**: `nu -c 'agentlint /Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/notfiles-config/claude/.claude/skills/pieces-health/SKILL.md /Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/notfiles-config/claude/.claude/skills/pieces-ltm/SKILL.md /Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/notfiles-config/claude/.claude/skills/pieces/SKILL.md'`

1. Copy the active `/Users/joe/.claude/skills/pieces-health/SKILL.md` into the new canonical worktree path, preserving its frontmatter and body before editing.
2. Add the explicit states `down`, `live-not-ready`, and `ready`.
3. State that HTTP 200 proves liveness only; readiness requires one bounded LTM call.
4. Remove advice that retries every repository or attributes direct Pieces MCP to personal-mcp.
5. Document restart as an operator action only after saturation is confirmed by CPU/RSS and a failed canary.
6. Commit all three canonical skill files: `docs(pieces): distinguish liveness from readiness`.
7. After merge, use Notfiles to replace the active standalone file with the canonical managed link; verify `readlink /Users/joe/.claude/skills/pieces-health/SKILL.md` resolves under `/Users/joe/.notfiles`.

### Task 5: Fix Devloop Clippy failure

**Crate**: `devloop`
**File(s)**: `/Users/joe/dev/devloop/crates/devloop/src/adapters/baml/council_analyzer.rs`
**Run**: `nu -c 'cd /Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/devloop; cargo clippy -p devloop -- -D warnings'`

1. Run Clippy; expected failure is `useless_borrows_in_formatting` at line 517.
2. Change `&insight.meta_summary` to `insight.meta_summary` in the `format!` arguments.
3. Run package nextest and Clippy; both must pass.
4. Commit: `fix(devloop): remove redundant synthesis borrow`.

### Task 6: Remove stale Devloop nextest override

**Crate**: `devloop`
**File(s)**: `/Users/joe/dev/devloop/.config/nextest.toml`
**Run**: `nu -c 'cd /Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/devloop; cargo nextest list'`

1. Confirm nextest warns about `profile.default.overrides.0.timeout` and no test matches `test_offline_mode`.
2. Delete only the override block for `test_offline_mode`; retain both TTY filters.
3. Run `cargo nextest list` and package nextest; the unknown-key warning must disappear.
4. Commit: `chore(devloop): remove stale nextest override`.

### Task 7: Require explicit Devkit live smoke tests

**Crate**: `devkit`
**File(s)**: `/Users/joe/dev/devkit/internal/ai/providers/router_test.go`
**Run**: `nu -c 'cd /Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/devkit; go test ./internal/ai/providers/... -run "^TestRouterSmoke$" -v -count=1'`

1. With unresolved provider variables present and no opt-in, confirm the current test attempts a live request.
2. Add this as the first code in `TestRouterSmoke`:
```go
if os.Getenv("DEVKIT_LIVE_TESTS") != "1" {
	t.Skip("set DEVKIT_LIVE_TESTS=1 to run live provider smoke tests")
}
````

3. Verify the default test skips. Verify an explicit opt-in with intentionally absent credentials reaches the existing credential skip, not the network.
4. Run `go test ./... -count=1` and commit: `fix(devkit): require opt-in for live provider smoke test`.

### Task 8: Add a rustfmt regression for taskit xtask generation

**Crate**: `taskit-init`
**File(s)**: `/Users/joe/dev/taskit/crates/taskit-init/src/scaffold.rs`
**Run**: `nu -c 'cd /Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/taskit; cargo nextest run -p taskit-init xtask_'`

1. Add a test that writes a fresh xtask, invokes `rustfmt --edition 2021 --check` on `xtask/src/main.rs`, and asserts success.
2. Run the focused test before changing templates; expected result: fail.
3. Commit the failing regression only: `test(taskit-init): require rustfmt-clean xtask scaffolds`.

### Task 9: Format taskit xtask templates

**Crate**: `taskit-init`
**File(s)**: `/Users/joe/dev/taskit/crates/taskit-init/src/scaffold.rs`
**Run**: `nu -c 'cd /Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/taskit; cargo nextest run -p taskit-init xtask_'`

1. Apply rustfmt's exact output to `XTASK_INJECT` and `XTASK_MAIN_FRESH` without changing generated behavior.
2. Run focused tests, workspace fmt, and Clippy.
3. Commit: `fix(taskit-init): generate formatted xtask source`.

### Task 10: Update the generated Notfiles xtask only

**Crate**: `notfiles-xtask`
**File(s)**: `/Users/joe/dev/notfiles/Cargo.toml`, `/Users/joe/dev/notfiles/xtask/Cargo.toml`, `/Users/joe/dev/notfiles/xtask/src/main.rs`
**Run**: `nu -c 'cd /Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/notfiles; cargo fmt --all -- --check'`

1. In the Notfiles worktree, add only `"xtask",` to the workspace member list; do not transfer the unrelated `notshell` edits from the primary checkout.
2. Copy `/Users/joe/dev/notfiles/xtask/Cargo.toml` and `/Users/joe/dev/notfiles/xtask/src/main.rs` into the same worktree paths. Do not copy `taskit.toml`, `Cargo.lock`, or any other dirty file.
3. Confirm only `xtask/src/main.rs` fails formatting, then run `rustfmt --edition 2021 xtask/src/main.rs`.
4. Run `cargo fmt --all -- --check`, `cargo clippy -p xtask -- -D warnings`, and `cargo check -p xtask`.
5. Commit the root manifest and `xtask/`: `chore(notfiles): adopt formatted xtask runner`.

### Task 11: Format Romp source and verify warm tests

**Crate**: `romp`
**File(s)**: `/Users/joe/dev/romp/src/main.rs`, `/Users/joe/dev/romp/src/proxy/mod.rs`, `/Users/joe/dev/romp/src/proxy/webhook.rs`
**Run**: `nu -c 'cd /Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/romp; cargo fmt -- --check'`

1. Confirm the three-file rustfmt diff.
2. Run `cargo fmt` and stage only those files.
3. Run Clippy, then nextest sequentially with a five-minute command timeout.
4. Commit: `chore(romp): apply canonical Rust formatting`.

### Task 12: Contain Gooey macro-generated missing-doc warnings

**Crate**: `baml-client`
**File(s)**: `/Users/joe/dev/gooey/baml_client/src/functions.rs`, `/Users/joe/dev/gooey/baml_client/src/providers.rs`
**Run**: `nu -c 'cd /Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/gooey; mise exec -- cargo clippy -p baml-client --lib -- -D warnings'`

1. Confirm exactly six missing-doc errors originate from `simplify_baml_macros` output.
2. Add `#![allow(missing_docs)]` at the top of each macro-heavy module and remove ineffective item-level missing-doc allows.
3. Do not regenerate BAML and do not edit Cargo registry sources.
4. Run package Clippy and tests; commit: `fix(baml-client): contain macro-generated docs lint`.

### Task 13: Add isolated Steve frontmatter regression tests

**Crate**: `steve`
**File(s)**: `/Users/joe/dev/steve/scripts/tests/test_fix_frontmatter.py`
**Run**: `nu -c 'cd /Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/steve; uv run pytest scripts/tests/test_fix_frontmatter.py -q'`

1. Add tests using `tmp_path` for heading-plus-paragraph extraction, no heading, malformed YAML, duplicate frontmatter blocks, and idempotent second execution.
2. Import both scripts by file path so the current syntax errors fail collection.
3. Run the file; expected result: collection failure.
4. Commit: `test(steve): cover frontmatter repair scripts`.

### Task 14: Repair Steve frontmatter script syntax

**Crate**: `steve`
**File(s)**: `/Users/joe/dev/steve/scripts/fix_frontmatter.py`, `/Users/joe/dev/steve/scripts/fix_frontmatter_strict.py`
**Run**: `nu -c 'cd /Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/steve; uv run pytest scripts/tests/test_fix_frontmatter.py -q'`

1. Replace both broken multiline literals with `r"^#+ .*\n\n(.+)"` and pass `re.MULTILINE` in both calls.
2. Expand the strict script's three semicolon-separated statements onto separate lines.
3. Run focused tests and `uv run ruff check scripts/fix_frontmatter.py scripts/fix_frontmatter_strict.py`.
4. Commit: `fix(steve): repair frontmatter description parsing`.

### Task 15: Split Steve blocking lint from debt inventory

**Crate**: `steve`
**File(s)**: `/Users/joe/dev/steve/Makefile`
**Run**: `nu -c 'cd /Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/steve; make lint-blocking'`

1. Add `lint-blocking` running `uv run ruff check --select E9,F63,F7,F82,S310,S602 steve scripts`.
2. Add `lint-full` running `uv run ruff check . --statistics` and keep it visibly non-green until debt is removed.
3. Make the default `lint` target call `lint-blocking`; do not add Ruff exclusions or unsafe auto-fixes.
4. Verify blocking lint and all tests pass; capture the 514-finding full inventory in the commit body.
5. Commit: `build(steve): separate blocking lint from debt inventory`.

### Task 16: Pin Inflection's required linter

**Crate**: `inflection`
**File(s)**: `/Users/joe/dev/inflection/mise.toml`, `/Users/joe/dev/inflection/Makefile`, `/Users/joe/dev/inflection/README.md`, `/Users/joe/dev/inflection/AGENTS.md`
**Run**: `nu -c 'cd /Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/inflection; mise exec -- golangci-lint version'`

1. Add `golangci-lint = "2.13.1"` under `[tools]`.
2. Make lint invoke `mise exec -- golangci-lint run ./...`; make `check-tools` fail when that exact tool is unavailable.
3. Document `mise install` and `mise run lint` in README and AGENTS.
4. Install only after GitHub API authentication works; then run lint, vet, and tests.
5. Commit: `build(inflection): pin required Go linter`.

### Task 17: Upgrade Magi SentencePiece

**Crate**: `magi`
**File(s)**: `/Users/joe/dev/magi/pyproject.toml`, `/Users/joe/dev/magi/uv.lock`, `/Users/joe/dev/magi/tests/test_deps_smoke.py`
**Run**: `nu -c 'cd /Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/magi; uv run pytest tests/test_deps_smoke.py -q'`

1. Add a smoke test importing `sentencepiece`, asserting version `>=0.2.2`, and constructing `SentencePieceProcessor()` without model or network access.
2. Run before changing dependencies; expected version assertion failure.
3. Change the dependency to `sentencepiece>=0.2.2,<0.3` and run `uv lock --upgrade-package sentencepiece`.
4. Run dependency smoke and full tests; confirm SWIG warnings disappear.
5. Commit: `fix(magi): upgrade SentencePiece bindings`.

### Task 18: Remove obsolete Doob Beads integration crates

**Crate**: `doob`
**File(s)**: `/Users/joe/dev/doob/crates/doob/tests/beads_adapter_test.rs`, `/Users/joe/dev/doob/crates/doob/tests/beads_integration_test.rs`
**Run**: `nu -c 'cd /Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/doob; cargo clippy --workspace --all-targets --all-features -- -D warnings'`

1. Confirm both files are disabled by undefined cfgs and reference removed APIs.
2. Delete both files; do not add `bd` or `integration-tests` features.
3. Run `doob-beads` tests and the all-target/all-feature workspace gate.
4. Commit: `test(doob): remove obsolete Beads integration crates`.

### Task 19: Isolate Doob sync test fixtures

**Crate**: `doob`
**File(s)**: `/Users/joe/dev/doob/crates/doob/tests/common/mod.rs`, `/Users/joe/dev/doob/crates/doob/tests/common/sync_mocks.rs`, `/Users/joe/dev/doob/crates/doob/tests/sync_domain_test.rs`, `/Users/joe/dev/doob/crates/doob/tests/sync_service_test.rs`
**Run**: `nu -c 'cd /Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/doob; cargo clippy --workspace --all-targets --all-features -- -D warnings'`

1. Remove `sync_mocks` from `common/mod.rs`.
2. Import it with `#[path = "common/sync_mocks.rs"] mod sync_mocks;` only in the two sync tests.
3. Remove unused `ProviderCapabilities` and `make_simple_todo` from `sync_mocks.rs`.
4. Run all-target Clippy and tests; commit: `test(doob): scope sync fixtures to consumers`.

### Task 20: Gate personal-mcp live process cleanup

**Crate**: `personal-mcp`
**File(s)**: `/Users/joe/dev/personal-mcp/src/proc_gc.rs`, `/Users/joe/dev/personal-mcp/src/main.rs`
**Run**: `nu -c 'cd /Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/personal-mcp; cargo nextest run proc_gc'`

1. Transfer only current selected work: save `git -C /Users/joe/dev/personal-mcp diff --binary -- src/main.rs` to `/tmp/personal-mcp-main.patch`, apply it in the worktree, and copy `/Users/joe/dev/personal-mcp/src/proc_gc.rs` to the worktree `src/proc_gc.rs`.
2. Add a failing test showing `dry_run=false` is denied when project mutations are disabled and no signal callback executes.
3. Before live candidate revalidation or signaling, call `require_project_mutations("proc_gc")`; leave preview and default dry-run ungated.
4. Put signaling behind a private injectable callback used by tests; production passes the existing signal implementation.
5. Update the MCP tool description to state that live cleanup requires mutation permission.
6. Run focused tests, full nextest, Clippy, and fmt; commit: `fix(personal-mcp): gate live process cleanup`.

### Task 21: Remove tracked build output from Tools

**Crate**: `tools`
**File(s)**: `/Users/joe/dev/tools/.gitignore`, `/Users/joe/dev/tools/AGENTS.md`, `/Users/joe/dev/tools/target/`, `/Users/joe/dev/tools/src/.DS_Store`
**Run**: `nu -c 'cd /Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/tools; let tracked = (git ls-files target | lines | length); if $tracked > 0 { exit 1 }'`

1. Run before cleanup; expected result: exit 1 with 3,321 tracked target paths.
2. Track `.gitignore` with `/target/`, `.DS_Store`, and `**/.DS_Store`.
3. Remove `target/` and tracked `.DS_Store` from the index, not from unrelated working files. Exclude `Cargo.lock` and `effort-report/`.
4. Add AGENTS.md with exact `cargo check`, `cargo clippy -- -D warnings`, and `cargo test` commands plus the artifact rule.
5. Verify no target path is tracked and run all Rust gates; commit: `chore(tools): stop tracking build artifacts`.

### Task 22: Commit Dumcp repository baseline documentation

**Crate**: `dumcp`
**File(s)**: `/Users/joe/dev/dumcp/.gitignore`, `/Users/joe/dev/dumcp/README.md`, `/Users/joe/dev/dumcp/AGENTS.md`
**Run**: `nu -c 'cd /Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/dumcp; go test ./... -count=1'`

1. Save `git -C /Users/joe/dev/dumcp diff --binary -- .gitignore` to `/tmp/dumcp-gitignore.patch`, apply it in the worktree, and copy `/Users/joe/dev/dumcp/README.md` to the worktree root.
2. Keep `.envrc` ignored and review README claims against `main.go`, `main_test.go`, and `go.mod`.
3. Add AGENTS.md documenting Go fmt/vet/test and filesystem-jail invariants from the README.
4. Run gofmt check, vet, and tests.
5. Commit: `docs(dumcp): add repository guidance`.

### Task 23: Clean Pieces Obsidian plugin metadata

**Crate**: `pieces-ob`
**File(s)**: `/Users/joe/dev/pieces-ob/.env.example`, `/Users/joe/dev/pieces-ob/.gitignore`, `/Users/joe/dev/pieces-ob/AGENTS.md`
**Run**: `nu -c 'cd /Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/pieces-ob; git diff --check'`

1. Save `git -C /Users/joe/dev/pieces-ob diff --binary -- .env.example` to `/tmp/pieces-ob-env.patch` and apply it in the worktree.
2. Keep the transferred environment wording edit.
3. Ignore `.DS_Store`, `**/.DS_Store`, and `.ctx/`; do not copy the primary checkout's `.DS_Store` or handoff stub.
4. Add AGENTS.md from CLAUDE.md with uv commands, PiecesOS readiness, Obsidian vault boundaries, and credential rules.
5. Run `glob scripts/*.py | each {|file| uv run python -m py_compile $file }` from the worktree and commit: `docs(pieces-ob): clarify local integration setup`.

### Task 24: Commit Goder environment hygiene

**Crate**: `goder`
**File(s)**: `/Users/joe/dev/goder/.gitignore`
**Run**: `nu -c 'cd /Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/goder; go test ./... -count=1'`

1. Save `git -C /Users/joe/dev/goder diff --binary -- .gitignore` to `/tmp/goder-gitignore.patch` and apply it in the worktree.
2. Confirm the only diff adds `.envrc` and that no secret file is tracked.
3. Run vet and tests.
4. Commit: `chore(goder): ignore local direnv configuration`.

### Task 25: Restore Devkit meta execution API and finish live-test gating

**Crate**: `devkit`
**File(s)**: `/Users/joe/dev/devkit/internal/ai/meta/exec.go`, `/Users/joe/dev/devkit/internal/ai/meta/exec_test.go`, `/Users/joe/dev/devkit/internal/ai/providers/router_test.go`
**Run**: `nu -c 'cd /Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/devkit; go test ./internal/ai/meta ./internal/ai/providers ./cmd/meta ./cmd/devkit -count=1'`

1. Add failing tests for `Exec` worker/summary output, `DEVKIT_LOG_DIR` commit log creation, no-summary output, and runner errors.
2. Implement `ExecResult{LogPath string}` and `Exec(context.Context, string, string, string, Runner, io.Writer, bool) (*ExecResult, error)` in `internal/ai/meta/exec.go` using `Run` plus the existing infra dev-log API.
3. Preserve the pending router smoke-test opt-in change; do not edit either CLI consumer.
4. Run focused and full Go tests, verify the branch, and commit the meta API as `fix(meta): restore CLI execution API`; commit the router test separately as `fix(devkit): require opt-in for live provider smoke test`.

### Task 26: Resolve Steve blocking security lint and commit lint policy

**Crate**: `steve`
**File(s)**: `/Users/joe/dev/steve/steve/skills/jira/scripts/jira-api.py`, `/Users/joe/dev/steve/steve/skills/testing/scripts/with-server.py`, `/Users/joe/dev/steve/scripts/tests/test_security_boundaries.py`, `/Users/joe/dev/steve/Makefile`
**Run**: `nu -c 'cd /Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/steve; make lint; uv run pytest scripts/tests -q'`

1. Add tests rejecting non-HTTPS, malformed, credential-bearing Jira origins and confirming the testing helper's shell command is documented as trusted local input.
2. Validate Jira origins before constructing authenticated requests; retain urllib only with a line-local S310 justification after validation.
3. Retain `shell=True` for documented `cd ... && ...` support and add a line-local S602 justification naming the trusted local CLI boundary.
4. Verify `make lint` and tests, then commit security boundaries and the pending Makefile together as `build(steve): enforce blocking security lint`.

### Task 27: Replace Inflection secrets environment mutation

**Crate**: `inflection`
**File(s)**: `/Users/joe/dev/inflection/internal/secrets/import_csv_test.go`, `/Users/joe/dev/inflection/internal/secrets/store_test.go`
**Run**: `nu -c 'cd /Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/inflection; mise exec -- go test ./internal/secrets; mise exec -- golangci-lint run --max-same-issues 0 --max-issues-per-linter 0 ./internal/secrets/...'`

1. Confirm uncapped lint reports 45 unchecked environment mutations.
2. Replace every set/cleanup pair with `t.Setenv`; use an empty value for missing-key cases.
3. Run focused tests and uncapped package lint; commit: `test(secrets): isolate environment state`.

### Task 28: Handle Inflection resource closes

**Crate**: `inflection`
**File(s)**: `/Users/joe/dev/inflection/internal/indexer/indexer.go`, `/Users/joe/dev/inflection/internal/sink/file_sink.go`
**Run**: `nu -c 'cd /Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/inflection; mise exec -- go test ./internal/indexer ./internal/sink'`

1. Reproduce seven errcheck findings.
2. For read/query resources, use the repository's `defer func() { _ = resource.Close() }()` convention. For writable sink files, return a close error when no earlier write error exists.
3. Run focused tests and uncapped lint; commit: `fix(storage): handle resource close results`.

### Task 29: Preserve Inflection viewer search count

**Crate**: `inflection`
**File(s)**: `/Users/joe/dev/inflection/internal/viewer/viewer.go`, `/Users/joe/dev/inflection/internal/viewer/viewer_test.go`
**Run**: `nu -c 'cd /Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/inflection; mise exec -- go test ./internal/viewer'`

1. Add tests proving `renderActiveContent` updates `searchCount` after source filtering and index errors.
2. Change `renderActiveContent` to a pointer receiver, convert the source chain to `switch`, and replace `WriteString(Sprintf(...))` with `fmt.Fprintf`.
3. Run focused tests and lint; commit: `fix(viewer): retain rendered search count`.

### Task 30: Expose Inflection version output

**Crate**: `inflection`
**File(s)**: `/Users/joe/dev/inflection/cmd/inflection/version.go`, `/Users/joe/dev/inflection/cmd/inflection/main.go`, `/Users/joe/dev/inflection/cmd/inflection/main_test.go`
**Run**: `nu -c 'cd /Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/inflection; mise exec -- go test ./cmd/inflection'`

1. Add tests for `-version` and `--version` that capture output and prove the TUI/config path is not entered.
2. Consume the linker-injected `version` variable in a version-only code path before configuration loading.
3. Run focused tests and lint; commit: `feat(cli): expose build version`.

### Task 31: Commit Inflection's pinned lint contract

**Crate**: `inflection`
**File(s)**: `/Users/joe/dev/inflection/mise.toml`, `/Users/joe/dev/inflection/Makefile`, `/Users/joe/dev/inflection/README.md`, `/Users/joe/dev/inflection/AGENTS.md`
**Run**: `nu -c 'cd /Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/inflection; mise exec -- gofmt -l cmd internal pkg; mise exec -- go test ./...; mise exec -- golangci-lint run --max-same-issues 0 --max-issues-per-linter 0 ./...'`

1. Verify formatting, tests, normal lint, and uncapped lint are all clean after Tasks 27-30.
2. Commit the existing pinned-tool and documentation changes: `build(inflection): pin required Go linter`.

### Task 32: Clear Doob baseline Clippy blockers and finish test cleanup

**Crate**: `doob`
**File(s)**: `/Users/joe/dev/doob/crates/doob-sync/src/service.rs`, `/Users/joe/dev/doob/crates/doobdash/src/ui.rs`, `/Users/joe/dev/doob/crates/doob/tests/beads_adapter_test.rs`, `/Users/joe/dev/doob/crates/doob/tests/beads_integration_test.rs`, `/Users/joe/dev/doob/crates/doob/tests/common/mod.rs`, `/Users/joe/dev/doob/crates/doob/tests/common/sync_mocks.rs`, `/Users/joe/dev/doob/crates/doob/tests/sync_domain_test.rs`, `/Users/joe/dev/doob/crates/doob/tests/sync_service_test.rs`
**Run**: `nu -c 'cd /Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/doob; cargo fmt --all --check; cargo clippy --workspace -- -D warnings; cargo nextest run --workspace'`

1. Replace the boolean equality assertion with `assert!` and remove the formatting borrow.
2. Commit those two baseline fixes as `test(doob): clear baseline Clippy blockers`.
3. Verify and commit the already-implemented obsolete-test deletion as `test(doob): remove obsolete Beads integration crates`.
4. Verify and commit the already-implemented fixture isolation as `test(doob): scope sync fixtures to consumers`.

### Task 33: Format Dumcp baseline and commit repository guidance

**Crate**: `dumcp`
**File(s)**: `/Users/joe/dev/dumcp/main_test.go`, `/Users/joe/dev/dumcp/.gitignore`, `/Users/joe/dev/dumcp/README.md`, `/Users/joe/dev/dumcp/AGENTS.md`
**Run**: `nu -c 'cd /Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/dumcp; gofmt -l main.go main_test.go; go vet ./...; go test -count=1 ./...'`

1. Apply gofmt to `main_test.go`; confirm the diff changes comment alignment only and commit `chore(dumcp): apply canonical Go formatting`.
2. Format README with the repository's Prettier hook, rerun Go gates, and commit the pending three documentation files as `docs(dumcp): add repository guidance`.

### Task 34: Clear Romp all-target Clippy

**Crate**: `romp`
**File(s)**: `/Users/joe/dev/romp/src/proxy/db_loggable.rs`
**Run**: `nu -c 'cd /Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/romp; cargo clippy --all-targets -- -D warnings; cargo nextest run --no-fail-fast'`

1. Reproduce `field_reassign_with_default` in `build_log_meta_from_headers`.
2. Initialize `ProxyHeaders` with `model_override`, `webhook_enabled`, `client_name`, and `..Default::default()` in one expression.
3. Run focused tests, all-target Clippy, and nextest; commit: `test(romp): initialize proxy headers directly`.

### Task 35: Remove remaining Doob test warnings

**Crate**: `doob`
**File(s)**: `/Users/joe/dev/doob/crates/doob/src/commands/update.rs`, `/Users/joe/dev/doob/crates/doob/tests/context_integration_test.rs`, `/Users/joe/dev/doob/crates/doob/tests/context_test.rs`
**Run**: `nu -c 'cd /Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/doob; cargo clippy --workspace --all-targets --all-features -- -D warnings'`

1. Rename the two double-underscore update tests with single underscores.
2. Replace both undeclared `no_parallel` cfg attributes with unconditional `#[serial_test::serial]`.
3. Remove only `mod common;` from `context_test.rs`; retain shared helpers for their 16 real consumers.
4. Run focused tests, all-target Clippy, and workspace nextest; commit: `test(doob): clear integration warning noise`.

### Task 36: Remove Gooey example warning noise

**Crate**: `baml-client`
**File(s)**: `/Users/joe/dev/gooey/baml_client/examples/productivity_insights.rs`, `/Users/joe/dev/gooey/baml_client/examples/comprehensive_metrics.rs`, `/Users/joe/dev/gooey/baml_client/examples/weekly_trends.rs`, `/Users/joe/dev/gooey/baml_client/examples/baml_transcript_analyzer.rs`, `/Users/joe/dev/gooey/baml_client/examples/analyze_hook.rs`, `/Users/joe/dev/gooey/baml_client/examples/pattern_analysis.rs`, `/Users/joe/dev/gooey/baml_client/examples/tool_patterns.rs`
**Run**: `nu -c 'cd /Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/gooey; cargo clippy -p baml-client --all-targets -- -D warnings'`

1. Remove only fields, methods, and imports proven unused by the context map.
2. Print the retained transcript session ID in its existing heading.
3. Store timestamps directly in pattern analysis; retain only tool strings in adjacent-pair analysis and correct its misleading 60-second comment.
4. Run example check, all-target Clippy, and package tests; commit: `chore(baml-client): remove example warning noise`.

### Task 37: Clear final Doob all-target Clippy findings

**Crate**: `doob`
**File(s)**: `/Users/joe/dev/doob/crates/doob/src/commands/handoff/sync.rs`
**Run**: `nu -c 'cd /Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/doob; cargo clippy --workspace --all-targets --all-features -- -D warnings'`

1. Replace the two test-local `vec!` values at lines 279 and 303 with arrays, preserving iteration behavior.
2. Run focused sync tests, all-target Clippy, fmt, and workspace nextest; commit: `test(doob): use fixed test fixtures`.

### Task 38: Clear final Gooey all-target Clippy findings

**Crate**: `baml-client`
**File(s)**: `/Users/joe/dev/gooey/baml_client/src/lib.rs`, `/Users/joe/dev/gooey/baml_client/tests/crate_tests.rs`, `/Users/joe/dev/gooey/baml_client/examples/transcript_analysis.rs`, `/Users/joe/dev/gooey/baml_client/examples/analyze_logs.rs`, `/Users/joe/dev/gooey/baml_client/examples/knowledge_base_builder.rs`
**Run**: `nu -c 'cd /Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/gooey; cargo clippy -p baml-client --all-targets -- -D warnings'`

1. Replace constant assertions with assertions that exercise exported crate metadata or remove redundant smoke tests when compilation is already the assertion.
2. Apply Clippy's behavior-preserving `Option::map`, let-chain, array-pattern, and `or_default` rewrites to the three examples.
3. Run example checks, all-target Clippy, fmt, and package nextest; commit: `chore(baml-client): satisfy all-target Clippy`.

### Task 39: Isolate Taskit pre-push Git environment

**Crate**: `taskit-engine`
**File(s)**: `/Users/joe/dev/taskit/.githooks/pre-push`, `/Users/joe/dev/taskit/crates/taskit-init/src/scaffold.rs`, `/Users/joe/dev/taskit/crates/taskit-engine/src/hooks.rs`
**Run**: `nu -c 'cd /Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/taskit-retry; cargo nextest run -p taskit-engine -p taskit-init'`

1. Add assertions that tracked and generated pre-push hooks clear Git's canonical local environment before invoking taskit.
2. Prepend `unset $(git rev-parse --local-env-vars)` to the tracked and generated pre-push bodies; do not mutate process-global Rust environment variables.
3. Reproduce the hook by pushing the integration branch normally; flow tests must create only their temporary repositories.
4. Run focused tests, fmt, Clippy, and pre-push; commit: `fix(hooks): isolate nested Git test repositories`.

## Deferred Operator Actions

- Restart PiecesOS once after stopping orchestration fan-out; verify health latency is under 100 ms and one 15-second LTM canary succeeds before enabling LTM again.
- Keep Braid `main` tracking `github/main`. Do not rename/remove Gitea remotes until the operator decides whether the mirror remains active.
- Preserve Doob's 19 modified files and `opencode.json`. Do not publish the misleading `feat/todo-add-due-flag` branch until those changes are classified.
- Do not remove Kan `.orig`, incomplete fuzz, Qlty, or worktree artifacts without a separate reviewed cleanup plan.
- Treat Steve's remaining full Ruff inventory as a follow-up program; never hide it with broad exclusions or apply the 88 unsafe fixes wholesale.
- Reconcile Obfsck HANDOFF files separately from the provider TODO; no production defect was found.

## Already-Resolved Verification

For Minibox, use the exact worktree and base commit in the setup table. Run `git merge-base --is-ancestor 3f226924 HEAD` and `cargo xtask verify`. The first command must exit 0 and the full gate must pass. Do not edit or commit source; the original trace is stale and no matching Minibox task ID exists to close.

## Verification Matrix

| Area             | Final command                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| ---------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Skills           | `agentlint /Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/notfiles-config/claude/.claude/skills/daily-orchestration/SKILL.md /Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/notfiles-config/claude/.claude/skills/pieces-health/SKILL.md /Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/notfiles-config/claude/.claude/skills/pieces-ltm/SKILL.md /Users/joe/dev/.worktrees/daily-orchestration-2026-08-26/notfiles-config/claude/.claude/skills/pieces/SKILL.md` |
| Devloop          | `cargo clippy -p devloop -- -D warnings`; `cargo nextest run -p devloop`                                                                                                                                                                                                                                                                                                                                                                                                                        |
| Devkit           | `go test ./... -count=1`                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| Taskit           | `cargo nextest run -p taskit-init`; `cargo clippy -p taskit-init -- -D warnings`                                                                                                                                                                                                                                                                                                                                                                                                                |
| Notfiles         | `cargo fmt --all -- --check`; project Clippy and tests                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| Romp             | `cargo fmt -- --check`; Clippy; nextest                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| Gooey            | `cargo clippy -p baml-client --all-targets -- -D warnings`; `cargo nextest run -p baml-client`                                                                                                                                                                                                                                                                                                                                                                                                  |
| Steve            | `make lint-blocking`; `uv run pytest --tb=short`; `make lint-full` remains an explicit debt report                                                                                                                                                                                                                                                                                                                                                                                              |
| Inflection       | `mise exec -- golangci-lint run ./...`; `go vet ./...`; `go test ./... -count=1`                                                                                                                                                                                                                                                                                                                                                                                                                |
| Magi             | `uv lock --check`; `uv run pytest tests/`                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| Doob             | `cargo clippy --workspace --all-targets --all-features -- -D warnings`; `cargo nextest run --workspace --all-features`                                                                                                                                                                                                                                                                                                                                                                          |
| personal-mcp     | `cargo nextest run`; `cargo clippy --all-targets -- -D warnings`                                                                                                                                                                                                                                                                                                                                                                                                                                |
| Tools            | `git ls-files target` returns empty; Rust gates pass                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| Go hygiene repos | `go vet ./...`; `go test ./... -count=1`                                                                                                                                                                                                                                                                                                                                                                                                                                                        |

## Pre-Save Checklist

- [x] Every confirmed failure maps to a task or an explicit already-fixed verification.
- [x] Every warning maps to a task, deferred operator action, or documented follow-up program.
- [x] Exact source paths come from the current context map.
- [x] No public API or serialization format changes are required.
- [x] Each implementation task has a focused failing check, verification command, and commit.
- [x] Existing dirty work is excluded from every planned commit.
