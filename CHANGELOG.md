## [0.8.0] - 2026-10-03

### Features

- _(crates)_ First publish of `godmode-core` and `godmode-cli` to crates.io

### Bug Fixes

- _(plan)_ Match relocated plans on task-id prefix; tighten relocation guards and dedupe the stem helper
- _(plan)_ Document relocated-plan identity and skip mechanism
- _(model)_ Move the plan-source existence check out of the data model
- _(godmode-core)_ Record pending tasks in the handoff snapshot
- _(skills)_ Repair Nushell parse errors in three helper scripts

### Misc

- _(plugin)_ Route generated agent/skill content through taskit instead of direct cargo

## [0.7.0] - 2026-10-02

### Features

- Config, handoff YAML writer, and session improvements
- _(hooks)_ Agent UX improvements — preamble lib, context cmd, auto-block, exit codes
- V0.6.0 — conformance tests, task add fix, preamble uses context cmd
- _(cli)_ Add --filter keyword support to task list, fixes #47
- _(godmode-core)_ Add tags field to Task for flexible grouping, fixes #49
- _(testing)_ Add reusable test infrastructure primitives
- _(model)_ Add completed_at timestamp to Task
- _(godmode-core)_ Starship statusline integration via cache file
- _(dispatch)_ Async dispatch loop for parallel execution fixes #56
- Add bats hook-test helpers to verification skill
- _(writing-plans)_ Recognise new doc filename conventions and infer plan doctype from path
- Differentiate idea/spec/plan doc types; brainstorm writes idea+spec, writing-plans writes plan
- Route specs to docs/specs/ and plans to docs/plans/ with path inference
- _(testing-philosophy)_ Add fuzz as sixth testing dimension
- _(skills)_ Add godmode:merge and godmode:decompose skills
- _(testing-philosophy)_ Add model checking with kani as 7th dimension
- _(memory-banking)_ Add skill with Rust CLI, templates, and lifecycle hooks
- _(skills)_ Add references, helpers, and active hooks to 8 incomplete skills
- _(hooks)_ Port 5 core hook scripts from nushell to rust
- _(hooks)_ Port session-start and post-write-plan-ingest to rust-script
- _(godmode-core)_ Add godmode init and doctor commands
- _(testing)_ Seven-dimension audit, property tests, and nu-to-Rust migration
- Add 13 skills, 36 agent configs, pipeline module, and gm command templates
- _(report-index)_ Add SOLID report index with incremental + rebuild
- _(pipeline)_ Add deterministic run_tasks executor and CLI integration
- _(agents)_ Add category field to all 36 agent configs
- Session pin, agent category prefix, and cd-guard hook
- _(governance)_ Add policy engine, SARIF output, and globstar verify step
- _(hooks)_ Port all skill hooks to compiled Rust
- _(xtask)_ Add cargo-xtask crate with pre-commit, ci, and dist gates
- _(agents)_ Add dialectic MoA pipeline + cliff.toml + moa-cas helper
- _(lifecycle)_ Add design skill, hook, and full lifecycle pipeline
- _(hooks)_ Add PostToolUse validators for JSON, TOML, YAML, and Nu syntax
- _(rustqual-workspace)_ Add helpers/triage.nu and references/workspace-toml-examples.md
- _(aichat)_ Add godmode assistant system pack
- _(skill)_ Agent-improvement-loop — seven-phase trace-to-code improvement cycle with coursers adapter
- _(rustqual-workspace/triage)_ Add --top, --only, --fix-config flags and COMPLEXITY surfacing
- _(ci)_ Add skill index completeness gate; fix 4 missing entries
- _(scripts)_ Add check-refs and gen-index utilities
- _(commands)_ Add 12 workflow gm- commands
- _(skills)_ Add doc-writer, doc-sync, doc-review skills
- _(commands)_ Add 5 MoA-identified workflow commands
- _(godmode)_ Improve cap and rust-conventions based on session patterns
- _(skills)_ Add crs-hook-testing skill — coursers rule lifecycle pipeline (gm3, JOB-472)
- Add Nushell completion generation
- _(godmode-core)_ Surface coursers failure-learning data in handon and context
- _(pipelines)_ Coursers-rules lifecycle pipeline with crs-\* skills (JOB-472)
- _(pipeline)_ Honor run entry points
- _(cli)_ Add task metadata options
- _(commands)_ Generate dual command projections
- _(hooks)_ Add registry coverage doctor
- _(workflow)_ Resume persisted execution
- _(pipeline)_ Execute optional and parallel steps
- _(doob)_ Publish local graph tasks
- _(todo)_ Add native issue synchronization
- _(eval)_ Add bounded skill evaluation subsystem
- _(commands)_ Derive workflows from pipelines
- _(pipelines)_ Add Crux reference gate
- _(pipelines)_ Merge Crux reference gate
- _(quality)_ Strengthen core release and conformance contracts
- _(quality)_ Merge core contracts migration
- _(trace)_ Crux-typed read layer for the trace log
- _(trace)_ Crux-typed emit API and a pure query layer
- _(cli)_ Godmode trace tail|failures|stats|summary
- _(config)_ Warn on unknown integration keys
- _(godmode-core)_ Parse session lifecycle markers as nameless records
- _(godmode-core)_ Mint session lifecycle markers at rotation
- _(godmode-core)_ Count markers with is_marker and add active_sessions
- _(godmode-cli)_ Omit marker-only sessions from trace summary
- _(godmode-cli)_ Label marker rows by event and report active_sessions
- _(godmode-core)_ Write handoff snapshot to tracked root HANDOFF.md

### Bug Fixes

- _(handoff)_ Correct doob project name, cleanup stale items, render HANDOFF.md
- _(handoff)_ Cleanup stale state file, update HANDOFF.yaml entries
- _(hooks)_ Pre-commit-gate uses JSON stdout decisions, not exit codes
- _(rx)_ Detect shell dynamically, improve error messages
- _(graph)_ Clean up depends_on on remove, return Result from next_task_id
- Add missing tags field to test constructor after merge
- _(session)_ Use completed_at for accurate duration_ms
- _(godmode-core)_ Auto-flush state on task transitions
- Add missing completed_at field to test constructor after merge
- _(gitignore)_ Re-ignore HANDOFF.godmode.\*.yaml state files
- _(skills)_ Introspection corrections — invalid CLI subcommands and tool hygiene
- Session quality improvements from session analysis
- _(pre-commit-gate)_ Parse command pipeline before checking for git commit
- _(skills)_ Add 7 missing skills to index, remove nonexistent godmode worktree subcommand
- _(hooks)_ Update stop-guard.nu for nu 0.106+ and remove invalid subcommand
- _(nu)_ Replace deprecated `get -i` with `get -o` in remaining scripts
- _(hooks)_ Fix stop hook failures on Linux
- _(skills)_ Introspect corrections — add memory-banking to skill table, remove bash heredocs
- _(skills)_ Rename memory-banking path to .ctx/memory-bank/
- _(skills)_ Add 14 missing skills to index, convert to JSON
- _(observability)_ 7 audit fixes for trace library and query helpers
- Update stale cruxx-core dep to crux-runtime
- _(skills)_ Remove stale helpers/fuzz/ reference from decompose
- _(hooks)_ Repair nu parse errors and deprecation in 3 hook scripts
- _(core)_ Correct lib.doctest key (was doctests)
- _(lifecycle)_ Code review fixes — brainstorm hook, lifecycle optional, design helpers
- _(hooks)_ Use try/catch for built-ins, fix nu --ide-check usage
- _(skills)_ Introspect corrections — branch guards, index sync, stale refs
- _(skills)_ Introspection corrections â remove nonexistent godmode report subcommand, fix audit.nu for nu 0.111+, add branch guard to refactoring, deduplicate issue-triage dispatch, remove phantom skill-index.json ref
- _(rustqual-workspace)_ Fix triage.nu for real rustqual JSON schema
- _(skills)_ Introspection corrections — add rustqual-workspace to index, fix using-crux handler-catalog ref
- _(godmode-trace)_ Extract named constants for time and calendar arithmetic
- _(skills)_ Add missing skills to index, add CLI subcommands to quick ref
- _(skills)_ Introspection corrections — replace cat/grep with Read/Grep tools, fix stale tdd-agent ref
- _(skills)_ Add memory-banking validation workflow with agentlint plugin
- _(commands)_ Remove nested commands/commands/ from cp artifact
- _(skills)_ Introspect corrections — remove stale 'not implemented' CLI disclaimers
- _(skills)_ Introspect corrections — add open-knowledge-write-skill to index, skip template placeholders in ref checker
- _(hooks)_ Remove 19 invalid godmode hook run entries
- _(skills)_ Introspection corrections — index gaps, doc drift, audit false positives
- _(observability)_ Wrap bare table expressions in print()
- _(skills)_ Introspection corrections — CLI accuracy, index refs, branch-guard clarity
- _(trace, verify)_ Trace-writer race, session-summary schema drift, crs-validate step
- _(skills)_ Introspection corrections — CLI docs gap, tool hygiene
- _(hooks)_ Wire agent-governance/parallel-agents into trace log
- _(devloop)_ Update CLI references to devloop git analyze, wire orphaned agents to skills
- _(skills)_ Introspection corrections — missing skill index entries, broken agent subcommand ref, undocumented CLI families
- _(commands)_ Correct broken skill/agent refs in gm/ commands, update README
- _(ci)_ Make conformance portable and refresh skill index
- _(commands)_ Preserve dual projection output
- _(commands)_ Roll back dual projection swaps
- _(quality)_ Enforce integration review contracts
- _(core)_ Preserve policy limits and restore conformance
- _(deps)_ Remove unused slashcrux dependencies
- _(trace)_ Rotate session_id and record sessions automatically
- _(trace)_ Warn when the resolved godmode cannot emit trace events
- _(memory-banking)_ Prefer canonical .ctx/memory-bank path
- _(plan)_ Keep task ids when a plan file is relocated

### Other

- Issue #47 — feat(cli): add --filter keyword support to task list
- Issue #49 — feat(model): add tags field to Task
- Issue #59 #60 — fix(rx): detect shell dynamically, improve error messages
- Issue #55 — perf(dispatch): use HashMap for O(1) task lookups
- Issue #54 #53 — fix(graph): clean up depends_on on remove, Result from next_task_id
- Issue #48 — raw JSON cleanup
- Issue #51 #52 — graph perf (done_ids cache + cycle detection)
- Issue #50 #57 — completed_at + duration tracking
- Issue #46 — starship status cache
- Issue #58 — crash-safe flush
- Issue #56 — async dispatch loop for parallel execution
- Issue #63 — bats hook-test helpers for verification skill
- Issue #69 — godmode init and doctor commands
- Issue/94-95 — extract named constants for time/calendar arithmetic (#94, #95)
- Issue/82-85 — add --top, --only, --fix-config flags and COMPLEXITY surfacing to triage.nu (#82, #83, #84, #85)
- Issue/87-93 — review.rs splits and godmode-core SRP module splits (#87-#93)
- Introspect corrections — stale 'not implemented' CLI disclaimers
- Introspect corrections — open-knowledge-write-skill index + template placeholder skip
- Gm3 coursers rule lifecycle skill (JOB-472)
- Coursers followups — trace-log wiring, crs-\* rule lifecycle pipeline (JOB-472)
- Issue #102
- Issue #104
- Prerequisite issue #102
- Prerequisite issue #104
- Issue #110
- Issue #106
- Issue #107
- Prerequisite issue #107
- Issue #111
- Issue #109
- Issue #108
- Issue #113
- Issue #112
- Integrate policy and conformance fixes
- Promote develop into staging
- Stage staging into release
- Finish release into main
- Sync main into develop
- Promote develop into staging
- Stage staging into release
- Finish release into main

### Refactor

- _(cli)_ Replace raw JSON strings with structured serialization
- Rewrite mini-context-graph skill to use kgx CLI
- Rename cruxx integration to crux across workspace
- _(quality)_ Fix rustqual findings — dedup, dead code, Display as_str, annotations
- _(review)_ Split check_skills/check_agents into focused helpers, reduce check_plugin_json complexity
- _(policy)_ Split policy.rs by SRP into loader / engine / types modules
- _(release)_ Split release.rs by SRP into version / tag / release modules
- _(sarif)_ Split sarif.rs by SRP into types / builder / entry modules
- _(templates)_ Split templates.rs by SRP into loader / render / apply modules
- _(cli)_ Split command dispatcher into handlers
- _(trace)_ Make \_lib/trace.nu a thin facade over godmode
- _(agents)_ Adopt domain-prefixed agent names and fix policy approval matching
- _(godmode-core)_ Put the HEAD sha behind a HeadSha port

### Documentation

- _(CLAUDE.md)_ Add config, context, cache, agent, testing module entries
- _(CLAUDE)_ Add subagent guardrails, git ops, CI watch, output style, and Nushell conventions
- _(mini-context-graph)_ Add kgx export commands to CLI reference
- Update all references from five/six to seven testing dimensions
- Update handoff
- Update handoff
- Update HANDOFF and seed kgx knowledge graph
- Rebuild agents/INDEX.md and add rustqual-workspace skill
- _(agent-improvement-loop)_ Reference updog standalone crate
- Update handoff
- _(memory-banking)_ Correct memory-bank path to .ctx/godmode/memory-bank/
- Add README for commands/gm, hooks, scripts, pipelines, workflows
- _(commands)_ Add 5 new workflow commands to README
- Update handoff
- _(skills)_ Add systematic-debugging feedback-loop reference

### Performance

- _(dispatch)_ Use HashMap for O(1) task lookups in independent_chains
- _(godmode-core)_ Cache done_ids in TaskGraph with RefCell
- _(godmode-core)_ Avoid O(n^2) path.clone() in would_create_cycle

### Testing

- Add property and conformance tests for skill and report_index
- Add property tests for Config TOML parsing
- _(pipeline)_ Cover non-entry-point runs
- _(cli)_ Cover task metadata validation
- _(trace)_ Property and integration coverage for the trace stack

### Miscellaneous Tasks

- Annotate remaining TODOs with GitHub issue numbers
- Update Cargo.lock after tokio dependency addition
- Update HANDOFF — pre-commit-gate pipeline fix
- Update handoff — nu-to-rust port planned
- Bump plugin version to d1e76d8
- Bump workspace version to 0.7.0
- Add commands/ dir (was ~/.claude/commands, now symlinked)
- Bump plugin version to 3700cb1
- Bump plugin version to e178504
- Bump plugin version to 5b513aa
- _(backlog)_ Track evidence-backed feature ideas
- Run nextest with all workspace features
- Audit Linux dependency graph
- Add gated release and bounded fuzz workflows
- Restore handoff metadata
- Checkpoint workspace and Rust documentation

## [0.6.1] - 2026-05-08

### Features

- _(hooks)_ Validate plugin.json version increments by at most 1 per component

## [0.6.0] - 2026-05-08

### Features

- _(skills)_ Add godmode:self-reflect — end-of-session retrospective skill
- _(commands)_ Add 6 workflow commands and pre-commit-gate hook; bump to v0.6.0

### Bug Fixes

- _(skills)_ Introspect blocking corrections — B1 todo-issue-sync flag, B2 refactoring skill ref, B3 task-runner invocation
- _(skills)_ Introspect nitpick corrections — N1 wave-integration frontmatter, N2 sed anti-pattern, N3 task filename
- _(skills)_ Introspect suggestion corrections — S1-S4 wave-integration, S5 cap frontmatter, S6 linear-mcp guard
- _(skills)_ Introspect corrections — blocking CLI ref + 4 nitpicks

### Miscellaneous Tasks

- Add inline TODO comments from codebase analysis

## [0.5.0] - 2026-05-08

### Bug Fixes

- _(dispatch)_ Apply exponential backoff in retry loop (#38)
- _(dispatch)_ Apply exponential backoff in retry loop (#38)
- _(dispatch)_ Add ChainOutcome::Skipped for slot-unavailable path (#39)
- _(dispatch)_ Add ChainOutcome::Skipped for slot-unavailable path (#39)
- _(cli)_ Check graphviz dot exit status in SVG path (#40)
- _(cli)_ Check graphviz dot exit status in SVG path (#40)
- _(observability)_ Instrument MoA helpers, harden trace append, fix trace_id separator (#42 #43 #44 #45 #41)

### Miscellaneous Tasks

- _(agents)_ Rename agent names to gm-\* prefix, normalize YAML indentation, add gm-orchestrator
- Bump version to v0.5.0

## [0.4.0] - 2026-05-04

### Features

- _(godmode-cli)_ Add --priority filter to task list and task next
- _(godmode-core)_ Add tracing spans to integrations
- _(godmode-core)_ Add WaveConfig, ConcurrencyTracker, health/retry primitives and gated dispatch
- _(godmode-core)_ Add task graph visualization via petgraph (to_petgraph, to_dot, visualize-graph CLI)

### Bug Fixes

- _(skills)_ Self-review corrections — align skill index
- _(skills)_ Self-review corrections — xtask, gh run watch, handoff yaml indent
- _(hooks)_ Guard CLAUDE_PLUGIN_ROOT and check GODMODE.tasks.yaml in session-start

### Miscellaneous Tasks

- Bump version to v0.4.0

## [0.3.2] - 2026-05-04

### Features

- _(model)_ Add Priority enum (High/Normal/Low) to Task
- Codify \_WORKING_DIR convention + Priority field + task-driven-development skill

## [0.3.1] - 2026-05-04

### Features

- _(session_trace)_ Wrap godmode session as Crux<TaskGraph> fixes #36
- Skill registry, YAML agents, and agent/skill CLI subcommands fixes #33
- Causal workflow system — command-driven DAGs per agent fixes #34
- _(hooks)_ Emit session.start/session.end trace events
- _(hooks)_ Replace fragile nu trace logic with rust-script
- _(skills)_ Add todo-issue-sync skill — audit TODOs and sync to GitHub/Linear issues

### Other

- Issue #36 feat(session_trace): wrap godmode session as Crux<TaskGraph>
- Issue #37 test(cruxx): add Session-based integration and conformance tests
- Issue #33 feat: skill registry, YAML agents, agent/skill CLI subcommands
- Issue #34 feat: causal workflow system — command-driven DAGs per agent

### Documentation

- Document session tracing hooks and godmode-trace.rs

### Testing

- _(cruxx)_ Add Session-based integration and conformance tests fixes #37

## [0.3.0] - 2026-05-03

### Features

- Initial godmode plugin — Rust-native methodology with task spine
- Add Rust workspace (godmode-core + godmode-cli), 19 tests passing
- Tool integration — --json, exit codes, hj/doob/rx/orca-strait
- Plugin registration, skill updates, integration tests, agent command
- Task execution, cruxx trace, doob sync, 3 new skills
- Task clear, plan ingest idempotency, task run --auto-done, godmode status, writing-plans docs
- Add tackle-issues slash command for parallel issue dispatch
- _(skills)_ Add cap, ci-fix, and wave-state orchestration
- _(skills)_ Add self-review skill for auditing skill consistency
- _(hooks)_ Align hooks.json with bazaar plugin conventions
- _(agents)_ Add valerie task management agent
- _(skills)_ Skill renames, helper fixes, and commands layer
- _(skills)_ Rename self-review → introspection, add report output, complete skill index; plan conformance test issues #13-17
- _(graph)_ Add unblock_all and task unblock-all CLI subcommand fixes #19
- _(graph)_ Cycle detection in graph::add fixes #18
- _(dispatch)_ Critical path calculation fixes #20
- _(hooks)_ Session-start triage hook fixes #7
- _(hooks)_ Pre-commit state validation fixes #8
- _(hooks)_ Auto-sync task-done to doob fixes #9
- _(skills)_ Add moa skill — parallel OpenAI proposers + synthesizer
- _(skills)_ Register wave-integration as godmode:wave-integration, add to index
- _(skills)_ Add wave-integration helpers and references
- _(godmode-core)_ Add verify module
- _(godmode-core)_ Add wave module
- _(godmode-core)_ Add worktree module
- _(godmode-core)_ Add ci_triage and issue_close to gh integration
- _(godmode)_ V0.2.0 — verify, wave, worktree, ci, issue subcommands
- _(integrations)_ Task pull --github adapter fixes #25
- _(skills)_ Add \_lib/quality-gate.nu
- _(skills)_ Add \_lib/guardrails.nu
- _(skills)_ Add \_lib/review-rules.nu
- _(skills)_ Add \_lib/dispatch.nu
- _(plan)_ Parse **Depends-on** annotation to override sequential deps fixes #26
- _(godmode-core)_ Task templates with var substitution
- _(godmode-core)_ Graph build command — interactive + file-driven
- _(godmode-core)_ Review + release subcommand groups
- Per-skill hook scripts, agents, and lifecycle hook wiring
- Hook observability, CLI status improvements, auto task IDs fixes #29 #31
- Godmode agent list, agent linter, INDEX.md generation fixes #30
- Changelog generation, hook migration system, version check fixes #32
- Wave 1 integration — hook observability, agent discoverability, CLI ergonomics, release hygiene
- _(cruxx)_ Real slashcrux integration — use StepState vocabulary for trace events
- _(conformance)_ Add typed conformance test harness — mirrors charmed_rust pattern
- _(conformance)_ Extend harness — wave, cruxx, fixture, plugin_structure, property tests + benchmarks
- _(godmode-core)_ Add started_at field to Task for duration tracking
- _(godmode-core)_ Add rx::list_scripts and rx::validate_run; promote which to runtime dep
- _(godmode-core)_ Introduce Session struct with duration tracking, cruxx trace writes, and rx validation
- _(godmode-cli)_ Wire task subcommands through Session; add graph_mut and unblock_all to Session

### Bug Fixes

- Strip unsupported fields from plugin.json manifest
- _(skills)_ Self-review corrections across six skill files
- _(agents)_ Remove stale godmode status ref and duplicate table row in valerie
- _(skills)_ Rename introspection → introspect, complete skill index
- _(skills)_ Self-review corrections — nextest consistency, stale heading, unblock-all, skill-index gap
- _(skills/wave-integration)_ Simplify helpers — remove dead state, fix proptest, align test commands
- _(skills)_ Add missing helpers files fixes #21
- _(skills)_ Add branch guard to ci-fix, tdd, writing-plans fixes #22
- _(cli)_ Task list --json exits 0 on empty graph fixes #23
- _(\_lib)_ Replace $now-ms variable with (now-ms) function call fixes #27
- _(\_lib)_ Replace dynamic use path with literal sibling import fixes #28
- _(ci)_ Build godmode-cli before conformance tests; fix helpers.nu run-external piping
- _(cruxx)_ Schema alignment with slashcrux vocabulary

### Other

- Issues #19 #18 #20 — unblock-all, cycle detection, critical path
- Issue #7 session-start triage hook
- Issue #8 pre-commit state validation
- Issues #9 #15 task-done-sync and Rust conformance
- Issues #10 #11 #12 \_lib helper extractions
- Issues #13 #14 #16 #17 conformance suite and CI
- _(godmode-core)_ Verify module (t1)
- _(godmode-core)_ Wave module (t2)
- _(godmode-core)_ Worktree module (t3)
- _(godmode-core)_ Gh ci_triage + issue_close (t4)
- Issue #21 missing helpers files
- Issue #22 branch guards in skills
- Issue #24 hook test harness
- Issue #23 #25 task list exit + gh adapter
- T1 \_lib/quality-gate.nu
- T2 \_lib/guardrails.nu
- T3 \_lib/review-rules.nu
- T4 \_lib/dispatch.nu
- T5 skill prose refs quality-gate + guardrails
- T6 skill prose refs review-rules
- T7 skill prose refs dispatch
- Issue #27 fix(\_lib) trace.nu hyphenated variable
- Issue #28 fix(\_lib) helpers.nu dynamic use path
- Issue #26 feat(plan) Depends-on annotation
- Hook observability + CLI ergonomics fixes #29 #31
- Agent discoverability + INDEX.md generation fixes #30

### Refactor

- _(\_lib)_ Extract open-trace helper fixes #10
- _(\_lib)_ Extract is-blocked helper fixes #11
- _(\_lib)_ Extract worktree-path helper fixes #12
- _(skills)_ Reference \_lib/quality-gate and guardrails in cap, ci-fix, tdd, refactoring, verification
- _(skills)_ Reference \_lib/review-rules in code-review and receiving-review
- _(skills)_ Reference \_lib/dispatch in parallel-agents and tackle-issues
- _(godmode-core)_ SOLID integration layer — subprocess helper, gh split, output types, rx tests
- _(skills)_ Extract \_lib functions, deduplicate skill prose
- _(cruxx)_ Replace TaskEvent with cruxx_core::Step constructors closes #35
- _(godmode-cli)_ Wire plan ingest and agent dispatch through Session

### Documentation

- Write README from real usage, add workflow section
- Update handoff
- Close gm-001 — define feature scope via roadmap issues
- Update handoff — session 3 close
- Update README — credit obra/superpowers, add new CLI commands
- Add references/ and helpers/ to all 11 skills
- Fix local path in install instructions
- Add CLAUDE.md with build commands and architecture reference
- _(readme)_ Update skill list, agents, commands, and helpers sections
- Add plan files for cycle-detection, unblock-all, critical-path; update .gitignore
- _(claude)_ Conformance crate commands, API gotchas, fmt-stage pattern

### Testing

- _(integrations)_ Add shell-out integration tests for hj and rx; fix stop hook prompt schema
- _(conformance)_ Rust CLI integration conformance fixes #15
- _(conformance)_ Plugin structure conformance suite fixes #13
- _(conformance)_ CLI subcommand conformance in SKILL.md files fixes #14
- _(conformance)_ Cross-skill consistency checks fixes #16
- _(hooks)_ Hook test harness for session-start, pre-commit, task-done-sync fixes #24
- _(conformance)_ Checks 13-14 — \_lib parse validation and reference integrity

### Miscellaneous Tasks

- Add target/ to .gitignore, remove from tracking
- Update handoff yaml, tdd-crate-agent, and valerie agent metadata
- Add tfc headless runner, update CLAUDE.md, skill edits, remove test-fix-commit
- Run conformance suite in GitHub Actions fixes #17
- Session handoff
- Add .version-bump.json for bump-version.sh
- Remove static nu conformance job — run locally via just conformance
- Bump version to 1.1.1
- Bump version to 1.2.0
- Bump version to 0.3.0
