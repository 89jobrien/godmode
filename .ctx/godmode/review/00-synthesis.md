# Five-lens review synthesis

Scope: all actionable observations from `01-api-surface.md`, `02-solid-modules.md`,
`03-test-coverage.md`, `04-blast-radius.md`, and `05-docs.md`, each of which reviewed the
validated complete `git diff main...HEAD` (469 files; 30,553 diff lines).

Severity is normalized to the requested rubric: **HIGH** for breaking changes,
compilation/rustdoc failures, security issues, or missing error-path tests; **MED** for SOLID
violations, missing happy-path coverage, or stale documentation; **LOW** for style, API
nitpicks, and non-critical documentation. `W1`-`W5` in fixes identify disjoint future
worktree slots described below.

| Priority | Finding | Lens | File:Line | Fix |
| --- | --- | --- | --- | --- |
| HIGH | Ten callable `godmode-conformance` APIs were removed, breaking path-dependent consumers. | API; blast radius | `tests/conformance/src/harness/context.rs:81`; `fixtures.rs:63,112`; `logging.rs:38`; `runner.rs:84-127` on `main` | **W1:** Restore deprecated forwarding methods, or explicitly version and document the test-harness break. |
| HIGH | Insight storage/report paths moved without fallback or migration; existing history becomes invisible. Operator docs also point to the wrong new report path. | API; blast radius; docs | `crates/godmode-core/src/insights.rs:39-136`; `AGENTS.md:72`; `CLAUDE.md:72` | **W1:** Read/merge legacy and canonical stores, migrate once without duplication, and update both operator guides to `.ctx/godmode/reports/insights/insights-YYYY-MM-DD.md`. |
| HIGH | Governance event names changed in place from `agent.start`/`agent.blocked` to `agent.approved`/`agent.denied`, breaking JSONL consumers. | API; blast radius | `crates/godmode-core/src/hooks/agent_governance.rs:42-54` | **W1:** Version the event schema or dual-emit/accept both vocabularies for a documented deprecation period. |
| HIGH | Four documented observability helper entry points were deleted, so path-based automation fails. | API | `skills/observability-as-infrastructure/helpers/{session-summary,trace-failures,trace-stats,trace-tail}.nu:1` on `main` | **W3:** Restore executable forwarding wrappers to `godmode trace`, with deprecation notices. |
| HIGH | The all-features rustdoc gate fails on two ambiguous links. | API | `crates/godmode-core/src/testing/mod.rs:12`; `testing/prop.rs:3` | **W1:** Use an explicit `mod@env` link and an unambiguous crate or macro target for `proptest`; rerun nightly rustdoc with warnings denied. |
| HIGH | `list_agents` changed from tolerant discovery to fail-fast parsing and can newly reject repositories that previously returned partial results. | API; blast radius | `crates/godmode-core/src/agent_index.rs:40,71-83` | **W1:** Preserve tolerant behavior in the existing API and expose strict validation separately, or document/version the behavioral break. |
| HIGH | Canonical command-source paths moved from `commands/gm/` to `command-support/gm/` without compatibility files. | API | `crates/godmode-core/src/command.rs:45-115`; removed `commands/gm/*.yaml` and `commands/gm/generate.nu` | **W4:** Add forwarding compatibility files or publish an explicit path migration with a breaking version. |
| HIGH | Context-map state moved from `.ctx/_WORKING_DIR` to `.ctx/godmode/_WORKING_DIR` with no fallback, so active legacy sessions are ignored. | API | `crates/godmode-core/src/hooks/context_map.rs:8-23` | **W1:** Check the canonical path first and legacy path second during a bounded migration window. |
| HIGH | `pre_commit::run` silently stopped stamping and staging the plugin manifest. | API | `crates/godmode-core/src/hooks/pre_commit.rs:25-38` | **W1:** Restore the side effect behind an explicit operation, or document/version the changed contract and update callers. |
| HIGH | Renamed dialectic agent artifact files have no aliases, breaking path-based launchers. | API | `agents/agent__dialectic-*.md` on `main`; `agents/dialectic-advocate.md:2` | **W3:** Add compatibility aliases/wrappers for old paths and migration notes. |
| HIGH | Hook command dispatch has no behavioral tests, including malformed input, subprocess failures, migration failures, and unknown built-ins. | Test coverage | `crates/godmode-cli/src/commands/hook.rs:10-439` | **W2:** Add `crates/godmode-cli/tests/hook_commands.rs` covering each command plus every identified parse/spawn/non-zero/unknown error path. |
| HIGH | The release orchestrator's mutating, retry, persistence, recovery, and cleanup error paths are untested. | Test coverage | `skills/rust-release-orchestrator/scripts/orchestrate.rs:186-416` | **W3:** Add fake-`cargo` integration tests for gate failures, remediation, exhausted/already-published retries, malformed/partial state, resume, reports, and cleanup. |
| HIGH | Skill lifecycle tracing is syntax-checked but lacks behavior tests for failures and malformed or missing state. | Test coverage | `hooks/scripts/pre-skill-trace.nu:9-22`; `post-skill-trace.nu:9-37` | **W3:** Add `hooks/tests/skill-trace.nu` for success/error pairs, malformed stdin, absent fields/marker, duplicate IDs, extraction, and cleanup. |
| HIGH | The TDD task-runner state machine lacks executable transition and failure-path tests. | Test coverage | `skills/task-driven-development/helpers/task-runner.rs:193-419` | **W3:** Add fixture-driven tests for dependencies, phases, three-attempt blocking, Cargo failures, UTF-8 titles, issue closure failure, and successful transitions. |
| HIGH | Extracted task-dispatch branches lack failure-path coverage. | Test coverage | `crates/godmode-cli/src/commands/task.rs:161-289` | **W2:** Test `Run`, non-GitHub `Pull`, `PushDone`, `UnblockAll`, `Apply`, and `ListTemplates`, including missing data/tools, child failures, duplicates, partial pushes, and invalid templates. |
| HIGH | OpenCode catalog/install error contracts are untested. | Test coverage | `crates/godmode-core/src/agent.rs:326-376` | **W1:** Add validation/read/write tests for IDs, paths, duplicates, empty/malformed catalogs, output creation, and per-file failures; **W2:** add the CLI install error cases. |
| HIGH | Command-renderer public error contracts are incompletely tested. | Test coverage | `crates/godmode-core/src/command.rs:46-197,266-269` | **W1:** Add table-driven source/YAML/name/template/projection/write/temp-cleanup failures; **W2:** cover missing output, missing `HOME`, `--check`, and dry-run CLI behavior. |
| HIGH | Trace-query public error paths and malformed-log behavior lack direct tests. | Test coverage | `crates/godmode-core/src/trace_stats.rs:78-233`; `crates/godmode-cli/src/main.rs:1994-2038` | **W1:** Add malformed/unreadable state and event-shape/boundary tests; **W2:** test contextual errors and human output. |
| HIGH | Context-map migration behavior and metadata failures are untested. | Test coverage | `crates/godmode-core/src/hooks/context_map.rs:13` | **W1:** Add tests for source filtering, missing/empty/current/stale maps, metadata failure, and legacy fallback behavior. |
| HIGH | Governance trace production lacks approved, denied, unknown-agent, and policy-resolution failure tests. | Test coverage | `crates/godmode-core/src/hooks/agent_governance.rs:44-56` | **W1:** Call the producer against isolated trace fixtures and assert both compatibility and canonical events, including failure fallback IDs. |
| HIGH | The depgraph report's subprocess, SARIF, persistence, and report-generation error paths are outside its test seam. | Test coverage | `skills/depgraph/helpers/depgraph-report.py:26-812` | **W3:** Move import-time work behind `main()` and test read/parse/escaping/snapshot/history/failure/cycle behavior plus a fake end-to-end command. |
| HIGH | The trace hook lifecycle and silent I/O/subprocess failures are untested. | Test coverage | `hooks/scripts/godmode-trace.rs:125-225` | **W3:** Add temporary-repository tests for start/end, malformed state, non-git fallback, unknown commands, append framing, I/O failure, and concurrent writers. |
| HIGH | Hardened-handoff checkpoint and repository-discovery error paths are largely untested. | Test coverage | `skills/handoff-hardened/helpers/checkpoint.nu:10-56`; `discover-dirty-repos.nu:5-52` | **W3:** Add isolated `HOME`/repo fixtures for corrupt state, defaults, statuses, dirty/upstream/ahead/behind/detached states, and failed Git commands. |
| HIGH | The xtask contract test checks source strings rather than command execution or failure propagation. | Test coverage | `xtask/tests/ci_contract.rs:3-17`; `xtask/src/main.rs:21-35,139-197` | **W5:** Add process tests with fake tools for ordering, reachability, argument splitting, root/`HOME`, copy failures, propagation, and unknown-command status. |
| MED | Hook auto-blocking mutates `TaskGraph` directly in the CLI, bypassing `Session` transition persistence and tracing. | SOLID | `crates/godmode-cli/src/commands/hook.rs:217-225` | **W2:** Move orchestration behind a core `Session::block_task` operation; leave only input/output translation in the CLI. |
| MED | The main CLI dispatcher still mixes all command families, filesystem work, orchestration, adapters, JSON, and rendering. | SOLID | `crates/godmode-cli/src/main.rs:950-2687` | **W2:** Extract one handler module per remaining command family and keep `main.rs` as parser/composition root. |
| MED | `Session` depends directly on graph, subprocess validation, tracing, filesystem persistence, and time. | SOLID | `crates/godmode-core/src/session.rs:12-16,119-177,233-260` | **W1:** Introduce focused graph-store, run-validator, clock, and trace-sink ports and inject a dependency bundle. |
| MED | Generic agent support now owns OpenCode-specific catalog, policy, rendering, installation, and writes. | SOLID | `crates/godmode-core/src/agent.rs:291-545` | **W1:** Move this into `agent::opencode`, separate pure rendering from installation, and move permission data out of the Rust match table. |
| MED | The command module mixes filesystem discovery/projection with pure validation and rendering. | SOLID | `crates/godmode-core/src/command.rs:45-281` | **W1:** Split model/renderer from filesystem source and projection adapters while retaining the public facade. |
| MED | Insight rendering constructs `JsonFileIndex` directly despite the `ReportIndexPort` abstraction. | SOLID | `crates/godmode-core/src/insights.rs:122-136`; `report_index.rs:54-67` | **W1:** Accept the port in orchestration and construct the JSON adapter only at the composition boundary. |
| MED | OpenCode installation writes directly into the live directory and can leave a mixed partial generation after failure. | Blast radius | `crates/godmode-core/src/agent.rs:356-376` | **W1:** Stage the complete set, validate it, atomically swap it into place, and retain the old generation on any failure. |
| MED | Empty/singleton and reduction-boundary depgraph layout behavior is not covered. | Test coverage | `skills/depgraph/helpers/depgraph_layout.py:6-9`; `test_depgraph_layout.py:6-17` | **W3:** Add zero/one-crate, one-ring, exact-boundary, and non-integral reduction cases. |
| MED | The `whatidid` architecture reference describes an obsolete Copilot/Python/GitHub Models pipeline and nonexistent pricing file. | Docs | `skills/whatidid/references/architecture.whatidid.md:5-98`; `skills/whatidid/SKILL.md:144` | **W3:** Rewrite it around Claude Code JSONL, Rust helpers, Anthropic API use, and the actual pricing location. |
| MED | Primary depgraph usage examples call a nonexistent script path. | Docs | `skills/depgraph/SKILL.md:34-56` | **W3:** Replace `xtask/scripts/depgraph-report.py` with `skills/depgraph/helpers/depgraph-report.py` and verify each command. |
| MED | The active daily-orchestration plan contains malformed, non-copyable fenced YAML examples. | Docs | `docs/plans/2026-08-26-daily-orchestration-health-remediation.md:248-262,322-327` | **W4:** Repair fence boundaries/backtick counts and nest `cmd` under `args`; validate the YAML snippets. |
| MED | Crux dependency examples use the invalid literal `<repository-url>` placeholder. | Docs | `skills/using-crux/SKILL.md:20-23,89-109` | **W3:** Insert the actual repository URL and verify the Cargo snippets resolve. |
| MED | The repository-health design falsely claims no new production public API. | Docs | `docs/designs/2026-09-06-repository-health-remediation-design.md:26-30` | **W4:** Record the exported command-rendering and trace-query modules/functions and their compatibility implications. |
| MED | The repository-health design falsely claims no trace-schema changes. | Docs | `docs/designs/2026-09-06-repository-health-remediation-design.md:71-77` | **W4:** Document governance event renames and the backward-compatible `active` status alias. |
| MED | The report index references six absent introspection reports. | Docs | `.ctx/godmode/reports/godmode-reports.index.json:21-34` | **W4:** Restore the six reports or remove the stale entries after checking provenance. |
| MED | Operator guides list six pipelines although nine definitions exist. | Docs | `AGENTS.md:193-212`; `CLAUDE.md:221-240` | **W4:** Add `aichat-system`, `coursers-rules`, and `lifecycle`, and update the count. |
| MED | Operator CLI inventories omit changed commands/options and existing top-level families. | Docs | `AGENTS.md:129-188`; `CLAUDE.md:130-194` | **W4:** Regenerate the inventories from current top-level and nested `--help` output. |
| MED | Core-module tables omit exported `command` and `trace_stats` modules. | Docs | `AGENTS.md:45-75`; `CLAUDE.md:45-75` | **W4:** Add both modules with concise ownership descriptions. |
| MED | The observability skill promises universal tracing that the helpers do not implement. | Docs | `skills/observability-as-infrastructure/SKILL.md:3-16` | **W3:** Narrow the guarantee to instrumented paths or add tracing to every covered helper and test it. |
| MED | The Crux pipeline README incorrectly says `crux-runtime` is already a local path dependency. | Docs | `pipelines/crux/README.md:20-21`; `Cargo.toml:23-24` | **W4:** State that the workspace uses crates.io `0.3.1` and that a sibling checkout is needed only for the CLI recipe. |
| MED | Generated command descriptions are truncated or malformed because fallback text is derived from prompt fragments. | Docs | `commands/gm-context.md:2`; `gm-dispatch-all.md:2`; `gm-review-code.md:2` | **W4:** Add explicit descriptions to canonical command YAML and regenerate projections. |
| MED | `AGENTS.md` overstates graceful degradation as universal across integrations. | Docs | `AGENTS.md:105-109`; `crates/godmode-core/src/integrations/gh/issues.rs:72-94`; `gh/ci.rs:99-130` | **W4:** Limit the claim to session-boundary best-effort calls and document propagating public paths. |
| LOW | New fallible APIs expose `anyhow::Error` and omit explicit `# Errors` contracts. | API | `crates/godmode-core/src/agent.rs:327-360`; `command.rs:46-115`; `agent_index.rs:123`; `skill.rs:42-48`; `testing/binary.rs:7`; `trace_stats.rs:78-211` | **W1:** Add `# Errors` docs now; introduce stable typed errors where callers need matching or recovery. |
| LOW | New public data structs expose all fields and are difficult to evolve compatibly. | API | `crates/godmode-core/src/agent.rs:293-323`; `command.rs:19-43`; `trace_stats.rs:12-68` | **W1:** Add `#[non_exhaustive]` with constructors/builders, or explicitly commit to field-layout stability. |
| LOW | `AgentConvergence.status` is an unconstrained `String` for a three-value domain. | API | `crates/godmode-core/src/trace_stats.rs:25-29` | **W1:** Replace it with a serialized enum and retain aliases if persisted values already exist. |
| LOW | Public mutation APIs use ambiguous bare `bool` dry-run switches. | API | `crates/godmode-core/src/agent.rs:356-360`; `command.rs:93-97` | **W1:** Introduce a shared `WriteMode`/`DryRun` enum or newtype. |
| LOW | OpenCode rendering exposes positional `(String, String)` tuples. | SOLID | `crates/godmode-core/src/agent.rs:339-374` | **W1:** Return a `RenderedAgent { file_name, content }` boundary type. |
| LOW | Core accepts `tail(..., limit = 0)` while the CLI rejects zero. | Blast radius | `crates/godmode-core/src/trace_stats.rs:78-86`; `crates/godmode-cli/src/main.rs:1018-1020` | **W1:** Define/document the core invariant; **W2:** align CLI behavior and add a direct regression test. |
| LOW | Two historical examples use nonexistent `/dev/godmode` paths. | Docs | `docs/plans/2026-05-01-plugin-tests-agent.md:28-30`; `2026-05-11-agent-ux-improvements.plan.md:62-66` | **W4:** Use a portable `$HOME/dev/godmode` example or clearly mark paths as illustrative. |
| LOW | Command-renderer APIs lack compiling usage examples. | Docs | `crates/godmode-core/src/command.rs:45-115` | **W1:** Add examples for validation, targets, dry-run writes, and stale-output checks. |
| LOW | OpenCode catalog/render/install APIs lack compiling usage examples. | Docs | `crates/godmode-core/src/agent.rs:326-360` | **W1:** Add examples for embedded/custom catalogs, rendering, and non-writing installation. |
| LOW | Trace-query APIs lack compiling usage examples. | Docs | `crates/godmode-core/src/trace_stats.rs:77-104,194-215` | **W1:** Add examples for session scoping and empty/malformed-log results. |
| LOW | Skill-index generation/check APIs lack compiling usage examples. | Docs | `crates/godmode-core/src/skill.rs:42-50` | **W1:** Add examples for the required `skills/` layout and stale-index result. |
| LOW | A changed generated agent file has an extra blank line at EOF. | Docs; blast radius | `agents/gov__observability-as-infrastructure-agent.md:56` | **W3:** Remove the extra blank line and rerun `git diff --check main...HEAD`. |

## Blocking

The **24 HIGH** findings above block merge under the requested rubric. They comprise ten
compatibility/rustdoc failures and fourteen missing error-path test groups.

## Suggestions

The **22 MED** findings above should be fixed or explicitly waived: seven architecture/runtime
design issues, one happy/edge-path test gap, and fourteen stale or broken documentation groups.

## Nitpicks

The **12 LOW** findings above are API ergonomics, consistency, examples, historical-doc cleanup,
and whitespace. Apply them in the same remediation pass where practical.

## Future non-overlapping worktree slots

1. **W1 — core and harness:** `crates/godmode-core/**` and
   `tests/conformance/src/harness/**`.
2. **W2 — CLI:** `crates/godmode-cli/**`.
3. **W3 — plugin executables/assets:** `hooks/**`, `skills/**`, and `agents/**`.
4. **W4 — documentation/projections/metadata:** root `AGENTS.md`/`CLAUDE.md`, `docs/**`,
   `pipelines/**`, `commands/**`, `command-support/**`, and
   `.ctx/godmode/reports/godmode-reports.index.json`.
5. **W5 — xtask:** `xtask/**`.

Where one finding names two slots, each slot owns only its listed files; no file ownership
overlaps. W2 integration tests may consume W1 behavior after integration without editing W1 files.

## Rejected, duplicate, and non-actionable observations

### Deduplicated into the table

- The insight migration was independently reported by API, blast-radius, and documentation
  lenses; one HIGH row includes both compatibility and operator-doc fixes.
- Removed conformance helpers appeared in API and blast-radius reports. The API report's symbol
  inventory establishes **ten**, not the blast-radius prose's “eleven”; the single HIGH row uses
  the enumerated count and resolves severity upward because removal is source-breaking.
- The governance event rename appeared in API and blast-radius reports; it is one HIGH schema
  compatibility finding. Its producer-test gap and stale design claim remain separate actions.
- The EOF whitespace issue appeared in blast-radius and documentation reports; it is one LOW row.
- OpenCode installation's partial-write defect and its missing write-failure test were not merged:
  one changes runtime behavior (MED), while the other closes an error-path test gap (HIGH).
- Context-map migration compatibility and context-map test coverage likewise remain separate
  because they require different deliverables.

### Rejected or non-actionable

- `FakeBinBuilder::stderr` was removed only from an integration-test-local helper and has no
  workspace callers; it is not a production/public API action.
- `run_hook_action` and `run_task_action` are syntactically `pub` but enclosed by a
  `pub(crate)` module and are not externally reachable. No API-break finding is retained; their
  behavioral test gaps are retained.
- Additive OpenCode, command, index, binary-path, and trace APIs compile at all known workspace
  call sites. No generic “new API” finding is retained beyond the specific contracts above.
- `release::validate_versions` now checking both plugin manifests is additive and covered; no
  compatibility action is required.
- The `Status` `active` deserialization alias is backward-compatible and serialization remains
  canonical; the false “no schema changes” design statement is still retained as stale docs.
- Documentation-only edits and the exhaustive changed-public-item/test maps are inventories, not
  defects, unless represented by a specific row above.
- Positive observations—one-way crate dependencies, improved policy/release/SARIF/template module
  ownership, successful workspace callers, and passing conformance checks—require no remediation.

## Compliance and evidence

- Inputs read completely: all five lens reports named above.
- Stack represented by the reports: Rust 2024/Cargo, Nushell, Markdown, YAML/JSON, Python, and
  shell helpers.
- Reported gates: `cargo check --workspace` passed; `cargo clippy --workspace --all-targets
  --all-features -- -D warnings` passed; `cargo nextest run --workspace` passed with 569 tests but
  reported one leaky test; `cargo xtask ci` passed; Rust and Nushell conformance passed; depgraph
  Python tests and hardened-handoff Nushell tests passed. Nightly all-features rustdoc failed on
  two links, and `git diff --check main...HEAD` failed on one blank EOF line.
- Synthesis tools: `skill`, `read`, `apply_patch`, `bash`, `personal_git_status`, and
  `personal_git_diff`. No source files were modified and no commit was created.

**Counts:** HIGH 24 · MED 22 · LOW 12 · **Total 58**.
