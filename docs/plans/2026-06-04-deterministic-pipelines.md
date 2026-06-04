# Deterministic Pipeline Integration

**Status**: approved
**Date**: 2026-06-04
**Depends on**: 2026-06-02-skill-pipelines.md (implemented)

## Problem

Pipelines define skill sequences but have no execution engine. Advancing requires
manual `godmode pipeline next` calls. The orchestrator agent has pipeline awareness
in its prompt but no deterministic mechanism to walk steps automatically.

## Design Principles

1. **Hooks never call `pipeline next`** — the boundary rule from the parent plan
   is inviolable. Hooks observe and annotate; the orchestrator or CLI drives state.
2. **Skills are not subprocesses** — skills are Claude Code prompts read by an
   agent session. A headless CLI cannot "invoke a skill." It can execute tasks.
3. **Tasks are the executable unit** — each task has an optional `run:` field
   resolved via `rx::resolve_cmd`. The CLI walks the task graph, not the skill graph.
4. **Three layers, distinct roles** — core executor (tasks), CLI (headless), hook
   (observability), orchestrator (interactive skill invocation).

## Architecture

```
┌──────────────────────────────────────────────────────┐
│                   gm-orchestrator                    │
│  Interactive sessions: reads SKILL.md, invokes       │
│  skills as Claude skills, calls pipeline next/skip   │
│  to advance. Handles optional step decisions.        │
│  THIS IS THE PRIMARY EXECUTION PATH.                 │
└──────────────┬───────────────────────────────────────┘
               │ calls
┌──────────────▼───────────────────────────────────────┐
│              godmode pipeline next/skip               │
│  State machine (pipeline.rs) — advance/skip/save     │
└──────────────┬───────────────────────────────────────┘
               │ used by
┌──────────────▼───────────────────────────────────────┐
│           godmode pipeline run <name>                 │
│  Headless mode: walks task graph per pipeline step.   │
│  For each step with matching tasks in the graph:      │
│    1. Find runnable tasks tagged with step's skill    │
│    2. Execute their run: fields via rx::run_cmd       │
│    3. Mark done on exit 0 (--auto-done semantics)     │
│    4. Advance pipeline state                          │
│  Skips steps with no matching tasks.                  │
│  Stops on first non-zero exit.                        │
└──────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────┐
│          post-pipeline-step hook                      │
│  OBSERVABILITY ONLY — fires after orchestrator or     │
│  CLI advances a step. Logs to session trace JSONL.    │
│  Does NOT call pipeline next. Does NOT mutate state.  │
└──────────────────────────────────────────────────────┘
```

## Layer 1: `pipeline::run_tasks`

New function in `crates/godmode-core/src/pipeline.rs`.

### Signature

```rust
/// Result of executing one pipeline step's tasks.
pub struct StepResult {
    pub skill: String,
    pub tasks_run: usize,
    pub tasks_failed: usize,
    pub skipped: bool,
}

/// Result of a full pipeline run.
pub struct RunResult {
    pub steps: Vec<StepResult>,
    pub completed: bool,
    pub stopped_at: Option<String>,
}

/// Walk the pipeline, executing task run: fields for each step.
///
/// For each step:
/// 1. Find tasks in the graph whose crate_name or title matches
///    the step's skill name (fuzzy: skill "task-driven-development"
///    matches tasks created during that skill's phase).
/// 2. If no matching tasks and step is optional: skip.
/// 3. If no matching tasks and step is required: skip (no tasks
///    to run — the step may be agent-only).
/// 4. For matching tasks with run: fields: execute via rx::run_cmd.
/// 5. On exit 0: mark task done. On non-zero: record failure.
/// 6. Advance pipeline state.
/// 7. If any task failed and fail_fast: stop and return.
///
/// loop:per-task steps repeat for all runnable tasks in the graph
/// (not just skill-matched ones), calling task next between iterations.
pub fn run_tasks(
    root: &Path,
    pipeline_name: &str,
    fail_fast: bool,
) -> Result<RunResult>
```

### Task-to-step matching

Tasks don't currently have a `skill` field. Matching strategy:

- Tasks created by `plan ingest` during a skill phase don't carry skill metadata.
- Instead of matching by skill name, `run_tasks` processes ALL runnable tasks
  per step, respecting dependency order via `graph::runnable()`.
- `loop:per-task` steps iterate: run one task, mark done, check next runnable,
  repeat until no runnable tasks remain or a task fails.
- Non-loop steps run all currently-runnable tasks in one batch.

This aligns with how the orchestrator works: it doesn't match tasks to skills
either — it calls `godmode task next` and works on whatever is runnable.

### State transitions

```
For each pipeline step:
  load graph -> runnable tasks -> execute run: fields
    -> mark done on success -> save graph
    -> advance pipeline state -> save pipeline state
    -> emit StepResult
```

## Layer 2: `godmode pipeline run`

New `PipelineAction::Run` variant in `main.rs`.

```
godmode pipeline run <name> [--from <skill>] [--fail-fast]
```

- Starts the pipeline (or resumes if already active with matching name)
- Calls `pipeline::run_tasks` with the given options
- Prints step-by-step progress to stdout
- Exits 0 if pipeline completes, 1 if stopped on failure
- `--json` emits `RunResult` as JSON

### Headless semantics

- Optional steps are skipped automatically (no human to decide)
- `parallel_with` steps are run sequentially (no agent dispatch available)
- Steps with no runnable tasks are advanced without action
- `loop:per-task` iterates until graph is drained or failure

## Layer 3: `post-pipeline-step` hook

Nushell or rust-script hook, registered as PostToolUse/Bash.

**Trigger**: Detects `godmode pipeline next` or `godmode pipeline skip` in the
command output.

**Action**: Appends a `pipeline.step.done` event to the session trace JSONL:

```json
{
  "event": "pipeline.step.done",
  "pipeline": "feature",
  "skill": "brainstorm",
  "step_index": 0,
  "timestamp": "2026-06-04T14:30:00Z"
}
```

**Boundary rule**: This hook MUST NOT call `godmode pipeline next`, `skip`, or
any other state-mutating command. It is observability infrastructure.

## Layer 4: gm-orchestrator refinements

The orchestrator prompt (updated in p5) already covers pipeline driving. Add:

1. **Deterministic mode flag**: When the user says "run the feature pipeline",
   the orchestrator should call `godmode pipeline run feature` for task-level
   work and only intervene for skill-level decisions (brainstorm prompts,
   code review judgment calls, verification checks).

2. **Hybrid execution**: For steps that are purely task-execution (like
   `task-driven-development` with `loop:per-task`), delegate to the CLI runner.
   For steps that require judgment (like `brainstorm`, `code-review`), invoke
   the skill directly as the active Claude skill.

3. **Resume semantics**: If the orchestrator's session ends mid-pipeline,
   `godmode handon` reports the active pipeline position. The next session's
   orchestrator picks up from `pipeline status`.

## Implementation

### Task: d1 — `pipeline::run_tasks`

New in `crates/godmode-core/src/pipeline.rs`:

- `StepResult`, `RunResult` structs
- `run_tasks(root, pipeline_name, fail_fast) -> Result<RunResult>`
- Uses `graph::load`, `graph::runnable`, `rx::run_cmd`, `graph::complete`
- Advances pipeline state after each step via `advance` + `save_state`
- Returns structured results, does no I/O formatting

### Task: d2 — CLI subcommand

Add `PipelineAction::Run` to `main.rs`:

- `godmode pipeline run <name> [--from <skill>] [--fail-fast]`
- Calls `pipeline::run_tasks`, formats output
- Supports `--json` for machine consumption

### Task: d3 — Observability hook

New `hooks/scripts/post-pipeline-step.rs` (rust-script):

- Parses Bash tool output for `pipeline next`/`pipeline skip` markers
- Appends trace event to `.ctx/godmode/sessions/YYYY-MM-DD.jsonl`
- Degrades silently if not in a pipeline or trace dir doesn't exist

### Task: d4 — Orchestrator prompt

Update `agents/prompts/gm-orchestrator.prompt.txt`:

- Add deterministic mode: delegate task-execution steps to CLI runner
- Add hybrid execution guidance
- Add resume semantics

### Task: d5 — This document

### Task: d6 — Tests

- Unit tests for `run_tasks` with in-memory graph (tempdir)
- Property test: pipeline always completes or stops at first failure
- Integration test: `godmode pipeline run` exits 0 on empty pipeline
- Verify hook does NOT call pipeline next (negative test)

## Out of Scope

- Task-to-skill metadata tagging (future enhancement)
- Parallel task execution within a step (use dispatch for that)
- Pipeline composition (pipeline calling another pipeline)
- Conditional branching in pipelines
