# HEAD b50f753 resolution audit

Audit target: committed `HEAD` `b50f753774655831089175979b9eb9c82e7e1a02` only. All source
evidence below was read from `git show HEAD:<path>`/`git grep ... HEAD`; the dirty working-tree
copies of `.kgx/**`, `.ctx/HANDOFF.godmode.godmode.yaml`, and
`crates/godmode-core/src/agent/opencode.rs` were excluded.

## Counts

| Original priority | RESOLVED | PARTIAL | BLOCKED | Total |
| --- | ---: | ---: | ---: | ---: |
| HIGH | 24 | 0 | 0 | 24 |
| MED | 22 | 0 | 0 | 22 |
| LOW | 12 | 0 | 0 | 12 |
| **Total** | **58** | **0** | **0** | **58** |

## HIGH findings

| ID | Status | Finding | Committed evidence |
| --- | --- | --- | --- |
| H01 | RESOLVED | Ten conformance harness APIs removed | Deprecated forwarders are restored in `tests/conformance/src/harness/context.rs:81`, `fixtures.rs:76,112`, `logging.rs:38`, and `runner.rs:84-127`; `harness/mod.rs:38` compiles and calls all ten in `deprecated_harness_forwarding_apis_remain_callable`. |
| H02 | RESOLVED | Insight path migration/history visibility | `insights.rs:89-139` merges and deduplicates canonical and legacy JSONL before migration; `:176-215` migrates legacy Markdown through an injected index; regression tests are `:313` and `:359`. Both operator guides use `.ctx/godmode/reports/insights/insights-YYYY-MM-DD.md` (`AGENTS.md:75`, `CLAUDE.md:75`). |
| H03 | RESOLVED | Governance event rename | `hooks/agent_governance.rs:44-65` emits both canonical `agent.approved`/`agent.denied` and legacy `agent.start`/`agent.blocked`; approval and denial regressions are at `:277` and `:294`. |
| H04 | RESOLVED | Removed observability helper entry points | Executable forwarding files exist at `skills/observability-as-infrastructure/helpers/{session-summary,trace-failures,trace-stats,trace-tail}.nu`, sharing `_forward.nu`; `tests/wrapper-tests.nu` exercises all four. |
| H05 | RESOLVED | All-features rustdoc ambiguity | Links are explicit (`testing/mod.rs:12` uses `mod@env`; `testing/prop.rs:3` uses an explicit macro target). `cargo +nightly rustdoc -p godmode-core --all-features -- -D warnings` passed in the isolated HEAD clone. |
| H06 | RESOLVED | `list_agents` became fail-fast | `agent_index.rs:40-59` preserves tolerant `list_agents` and adds separate `list_agents_strict`; `:436-449` verifies malformed authoritative cfg is tolerated only by the legacy API. |
| H07 | RESOLVED | `commands/gm/` compatibility path removed | HEAD contains the complete `commands/gm/*.yaml` source mirror and `commands/gm/generate.nu`; projection conformance `command_projections_match_all_canonical_sources` passed in nextest. |
| H08 | RESOLVED | Context-map legacy sessions ignored | `hooks/context_map.rs:13-18` prefers `.ctx/godmode/_WORKING_DIR` and falls back to `.ctx/_WORKING_DIR`; `:100` verifies a recent legacy map. |
| H09 | RESOLVED | Pre-commit manifest mutation contract changed silently | `hooks/pre_commit.rs:106-140` exposes and documents the explicit compatibility operation `stamp_plugin_manifest` rather than silently hashing/staging in `run`; `:219` verifies the explicit operation. |
| H10 | RESOLVED | Dialectic artifact aliases absent | Five old-path aliases are committed as `agents/agent__dialectic-{advocate,alternative,orchestrator,skeptic,synthesizer}.md`; discovery deduplication is tested by `agent_index.rs:382`. |
| H11 | RESOLVED | Hook command behavior untested | `crates/godmode-cli/tests/hook_commands.rs:25-181` covers list/log/test/migrate/run, malformed input, read/spawn/non-zero failures, unknown built-ins, ordering, and persisted auto-blocking. All six tests passed in the isolated nextest run. |
| H12 | RESOLVED | Release-orchestrator mutation/recovery tests | `skills/rust-release-orchestrator/tests/test_orchestrate.py:29-63` implements fake cargo; at `:47-50` it emits an already-uploaded stderr message and exits nonzero (`101`) when `ALREADY_PUBLISHED_ADAPTER=1`. `test_already_published_response_is_success_without_retry` at `:126-134` asserts orchestrator success, one publish invocation, the explicit nonzero-error marker, and state cleanup. Tests at `:85,105,113` cover remediation, exhausted/retried publish, persistence, malformed/partial state, resume, reports, and cleanup. All four integration tests passed. |
| H13 | RESOLVED | Skill lifecycle behavior tests | `hooks/tests/skill-trace-tests.nu:1-56` exercises success/error pairs, malformed stdin, missing skill/ID/marker, duplicate IDs, stderr extraction, and marker cleanup against `pre-skill-trace.nu` and `post-skill-trace.nu`. The suite passed in the isolated HEAD clone. |
| H14 | RESOLVED | TDD task-runner state-machine tests | `skills/task-driven-development/tests/test_task_runner.py:63-132` covers dependency/phase transitions, malformed and missing state, Cargo failure, three-attempt blocking (`:109`), UTF-8 titles (`:120`), issue-close failure (`:126`), and successful red/green/refactor progression. All seven tests passed. |
| H15 | RESOLVED | Extracted task-dispatch failure coverage | `crates/godmode-cli/tests/task_dispatch.rs:34-279` exercises all six branches plus missing task/run/tool, non-zero and signal exits, duplicate and empty/invalid pull data (`:189`), partial push, empty links, missing/invalid templates, and missing variables. All nine tests passed in nextest. |
| H16 | RESOLVED | OpenCode catalog/install errors untested | Catalog validation/read cases are covered at `agent/opencode.rs:754-813,874,900`; empty/router-only catalogs and staged/per-file/swap/transaction failures are covered at `:615-680`; CLI catalog/`HOME`/output failures remain at `crates/godmode-cli/tests/command_renderer.rs:141-171`. These tests passed in nextest. |
| H17 | RESOLVED | Command-renderer error contracts | Core tests `command.rs:455-624` cover names, source/read/YAML/prompt/template/duplicate/projection/write/stale/preview cases; `projection.rs` tests transaction rollback; CLI tests `command_renderer.rs:64-139` cover output, `HOME`, check, dry-run, and surfaced renderer errors. These passed in nextest. |
| H18 | RESOLVED | Trace-query errors/malformed logs untested | Core `trace_stats.rs:599-642` covers zero and I/O errors; CLI `tests/trace_queries.rs:30-242` covers malformed/legacy rows, session boundaries (`:88`), unreadable/malformed state (`:143,163`), contextual errors (`:198`), zero (`:182`), and human output (`:214`). These passed in nextest. |
| H19 | RESOLVED | Context-map migration/metadata tests | `hooks/context_map.rs:59-115` covers non-source filtering, missing/empty/current/stale canonical maps, entry-metadata failure, legacy fallback, and scratch read failure. All five tests passed in nextest. |
| H20 | RESOLVED | Governance trace producer tests | `hooks/agent_governance.rs:250-318` covers unknown fallback IDs, policy-resolution failure reason, approved/denied canonical and legacy events, and non-fatal trace-write failure. All five tests passed in nextest. |
| H21 | RESOLVED | Depgraph report outside test seam | Import work is behind `main`; `test_depgraph_report.py:31-121` covers import safety, malformed SARIF, counts/escaping, snapshots/history, subprocess failures, empty/singleton E2E, cycles, and propagation. The corrected test invocation passed 15 depgraph tests. |
| H22 | RESOLVED | Trace-hook lifecycle/I/O tests | `hooks/scripts/godmode-trace.rs:135-216` covers start/end, malformed state, non-git fallback, unknown command, directory/open/write failures, JSONL framing, and concurrent writers. `rust-script --test hooks/scripts/godmode-trace.rs` passed 8/8. |
| H23 | RESOLVED | Hardened-handoff checkpoint/discovery errors | `skills/handoff-hardened/tests/checkpoint-tests.nu:1-48` covers corrupt/default/stale/current checkpoints and clean/dirty/ahead/behind/status-failure/detached/upstream/rev-list/invalid-count states through `classify-repo`; the Nushell suite passed. |
| H24 | RESOLVED | xtask source-string contract | `xtask/tests/ci_contract.rs:117-343` executes fake tools and verifies ordering, reachability, split args, root, `HOME`, copy failure, child-status propagation, and unknown-command status. All eight process tests passed in nextest. |

## MED findings

| ID | Status | Finding | Committed evidence |
| --- | --- | --- | --- |
| M01 | RESOLVED | Hook auto-block bypassed `Session` | `crates/godmode-cli/src/commands/hook.rs:245` calls `Session::block_task`; persistence is exercised by `tests/hook_commands.rs:158` and core `session.rs:793`. |
| M02 | RESOLVED | Main CLI dispatcher mixed all concerns | `crates/godmode-cli/src/main.rs:819` delegates to `commands::dispatch`; family modules include `agent.rs`, `command.rs`, `session.rs`, `trace.rs`, and `workspace.rs`. `tests/cli_boundary.rs` enforces the boundary. |
| M03 | RESOLVED | `Session` directly depended on infrastructure/time | `session.rs:23-151` defines and injects graph-store, run-validator, clock, and trace-sink ports; transitions and summaries use `ports.clock` at `:290,305,375,407,425`. `session_routes_transition_and_summary_time_through_clock_port` at `:926` verifies deterministic timing. |
| M04 | RESOLVED | Generic agent module owned OpenCode policy/install | Generic `agent.rs:10-16` now only declares `opencode` and re-exports its public compatibility surface; all OpenCode types, rendering, validation, installation, and filesystem mechanics are owned by `agent/opencode.rs:12-522`. Permission mappings remain data in `agents/opencode-projects.yaml:5-30`. `generic_agent_module_does_not_implement_opencode_catalog` at `agent.rs:322-340` enforces the thin facade, and passed in nextest. |
| M05 | RESOLVED | Command module mixed pure and filesystem work | `command.rs:9-10` splits `fs` and `render` adapters while retaining the public facade; `render_commands_with_templates` is pure and tested at `command.rs:572,583`. |
| M06 | RESOLVED | Insight rendering constructed concrete index | `insights.rs:176-181` accepts `&dyn ReportIndexPort`; `:359` verifies an injected recording port. The compatibility facade alone constructs `JsonFileIndex` at `:157-159`. |
| M07 | RESOLVED | OpenCode live-directory partial writes | `agent/opencode.rs:257-381` stages, backs up, swaps, restores, and cleans up transaction state; injected-failure tests at `:635-680` cover per-file failure, old-generation restoration, and stale transaction artifacts. |
| M08 | RESOLVED | Depgraph layout boundary coverage | `test_depgraph_layout.py:7-47` covers empty/singleton, one ring, shallow/deep boundaries, zero edges, non-integral reduction, and invalid boundaries. |
| M09 | RESOLVED | Obsolete whatidid architecture | `skills/whatidid/references/architecture.whatidid.md:10-73` now documents Claude Code JSONL, Rust helpers, Anthropic headers, and `skills/whatidid/model_pricing.json`. |
| M10 | RESOLVED | Invalid depgraph script examples | All primary examples use `$CLAUDE_PLUGIN_ROOT/skills/depgraph/helpers/depgraph-report.py` (`skills/depgraph/SKILL.md:38-47`). |
| M11 | RESOLVED | Malformed daily-orchestration YAML | The corrected example nests `cmd` beneath `args` at `docs/plans/2026-08-26-daily-orchestration-health-remediation.md:255-256`; fence structure is balanced in committed content. |
| M12 | RESOLVED | Invalid Crux repository placeholder | `skills/using-crux/SKILL.md:23,92,99,106` uses `https://github.com/89jobrien/crux`. |
| M13 | RESOLVED | Design denied new public API | `docs/designs/2026-09-06-repository-health-remediation-design.md:24-37` inventories command, report-index, and trace-query APIs. |
| M14 | RESOLVED | Design denied trace-schema changes | The design documents canonical/legacy status and consumer-facing governance fields at `:39-53` and records persisted-state changes at `:99`. |
| M15 | RESOLVED | Report index referenced absent reports | `.ctx/godmode/reports/godmode-reports.index.json` now lists only `introspection-2026-09-06.md`; the six absent entries are removed. |
| M16 | RESOLVED | Operator guides listed six pipelines | Both guides list nine pipelines including `lifecycle`, `aichat-system`, and `coursers-rules` (`AGENTS.md:190-200`, `CLAUDE.md:191-201`). |
| M17 | RESOLVED | CLI inventories stale/incomplete | `AGENTS.md:129-185` and `CLAUDE.md:130-186` contain the regenerated top-level/nested command inventory; conformance test `operator_docs_match_workspace_and_cli_surface` passed. |
| M18 | RESOLVED | Core tables omitted modules | `command` and `trace_stats` are documented at `AGENTS.md:68,70` and `CLAUDE.md:68,70`. |
| M19 | RESOLVED | Observability promised universal tracing | `skills/observability-as-infrastructure/SKILL.md:4-16` limits the contract to instrumented helpers and registered lifecycle hooks. |
| M20 | RESOLVED | Crux README claimed local path dependency | `pipelines/crux/README.md:20-25` identifies crates.io `crux-runtime` `0.3.1` and scopes sibling checkout use to the CLI recipe. |
| M21 | RESOLVED | Generated command descriptions malformed | Canonical descriptions are explicit (`command-support/gm/{context,dispatch-all,review-code}.yaml:2`) and projections carry the same text (`commands/gm-{context,dispatch-all,review-code}.md:2`); projection conformance passed. |
| M22 | RESOLVED | AGENTS overstated universal graceful degradation | `AGENTS.md:109-115` and `CLAUDE.md:109-115` distinguish best-effort session calls from propagating public integration paths. |

## LOW findings

| ID | Status | Finding | Committed evidence |
| --- | --- | --- | --- |
| L01 | RESOLVED | Fallible APIs lacked stable errors/`# Errors` | Every named fallible surface documents errors: generic agent APIs at `agent.rs:104,115,154,254,284`; OpenCode APIs at `agent/opencode.rs:136,213,242`; `command.rs:101,151,191,222,249,266`; `agent_index.rs:41,50,146,176`; `skill.rs:18,47,77`; `testing/binary.rs:8`; and `trace_stats.rs:227,251,267,298,402,434`. Existing `anyhow::Result` facades remain source-compatible. |
| L02 | RESOLVED | Public data structs difficult to evolve | OpenCode structs are `#[non_exhaustive]` with constructors/builders in `agent/opencode.rs:12-129`; command structs remain protected at `command.rs:23-98`; all four trace result structs are `#[non_exhaustive]` with constructors/builders at `trace_stats.rs:12-218`, tested at `:581`. |
| L03 | RESOLVED | Convergence status was a `String` | `trace_stats.rs:39-86` defines serialized `ConvergenceStatus`; `AgentConvergence.status` uses it at `:95`; compatibility is tested at `:620`. |
| L04 | RESOLVED | Mutation APIs used bare booleans | Shared `WriteMode` is defined in `write_mode.rs:5-29`; explicit-mode APIs are used by command, OpenCode, and session pruning while bool facades remain source-compatible. |
| L05 | RESOLVED | OpenCode renderer returned positional tuples | `agent/opencode.rs:169-212` defines `RenderedAgent`, exposes typed rendering, and retains the tuple API only as a compatibility wrapper; `agent.rs:12-16` re-exports the typed facade. Test: `agent/opencode.rs:866`. |
| L06 | RESOLVED | Core/CLI zero-tail mismatch | Core documents/returns empty at `trace_stats.rs:227-246`; CLI accepts the same invariant and `tests/trace_queries.rs:182` verifies it. |
| L07 | RESOLVED | Historical `/dev/godmode` examples | Neither committed plan contains `/dev/godmode`; `docs/plans/2026-05-01-plugin-tests-agent.md:30` uses `$GODMODE_ROOT`, and the obsolete absolute-path example was removed from `2026-05-11-agent-ux-improvements.plan.md`. |
| L08 | RESOLVED | Command APIs lacked compiling examples | Compiling examples cover load, target rendering, write/preview, and stale-output checks in `command.rs:99-293`; nightly rustdoc passed. |
| L09 | RESOLVED | OpenCode APIs lacked compiling examples | Catalog, render, and non-writing/install examples are in the owning module at `agent/opencode.rs:136-260`; nightly rustdoc passed. |
| L10 | RESOLVED | Trace-query APIs lacked compiling examples | Session-scoped and empty/malformed-safe contracts are documented at `trace_stats.rs:227-460`; nightly rustdoc passed. |
| L11 | RESOLVED | Skill-index APIs lacked examples | `skill.rs:47-101` contains compiling layout/generation/check examples with explicit errors; nightly rustdoc passed. |
| L12 | RESOLVED | Extra blank line in generated agent | `agents/gov__observability-as-infrastructure-agent.md` no longer has the extra EOF line; `git diff --check dbbed8d..HEAD` passed. |

## Partial or blocked rows

- **PARTIAL:** none.
- **BLOCKED:** none.

## Commit and worktree verification

- Fix commits `e6c30f7`, `041df21`, `839ddf0`, `6144222`, and `6ac7fd4` are ancestors of
  HEAD (`git merge-base --is-ancestor` returned 0 for each).
- Integration merges are a sequential first-parent chain and are all ancestors of HEAD:
  `70d5803` (second parent `e6c30f7`) → `3f35616` (`041df21`) → `37404e5` (`839ddf0`) →
  `44c193b` (`6144222`) → `6226044` (`6ac7fd4`).
- Follow-up fixes `d456f25` and `d0c16c7` are ancestors of HEAD. Their integration merges form
  the subsequent first-parent chain: `e0d342f` (second parent `d456f25`) → `1321d24` (second
  parent `d0c16c7`).
- Third-attempt fixes `adfc159` and `8e6e865` are ancestors of HEAD. Their sequential integration
  merges continue the first-parent chain: `6762e85` (second parent `adfc159`) → `b50f753`
  (second parent `8e6e865`).
- `git worktree list --porcelain` contains no worktree whose branch is `moa-review/*`; all six
  review worktrees are removed. The unrelated primary, `fix/moa-integration`, and prunable
  pre-existing worktrees were not assessed or modified.

## Delivery gates and tools

- PASS: `cargo fmt --all --check`.
- PASS: standard `cargo clippy --workspace -- -D warnings`.
- PASS: `cargo nextest run --workspace` — 644/644 passed, 0 skipped.
- PASS: `cargo +nightly rustdoc -p godmode-core --all-features -- -D warnings`.
- PASS: conformance runner — 52/52.
- PASS: release-orchestrator Python integration tests — 4/4; task-runner Python tests — 7/7;
  depgraph Python tests — 15/15;
  skill-trace, handoff-checkpoint, and observability-wrapper Nushell suites.
- PASS: trace-hook embedded Rust tests — 8/8.
- PASS: `git diff --check dbbed8d..HEAD` (full original and follow-up remediation range).
- Tools used: `skill`, `read`, `grep`, `bash`, `personal_git_status`, and `apply_patch`.

No source file or commit was modified; only this gitignored audit report was written.
