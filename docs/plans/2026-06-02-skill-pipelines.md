# Skill Pipelines — Chain Edges and Named Workflows

**Status**: approved
**Date**: 2026-06-02

## Problem

Godmode has 42 skills and 35 agents but no declared ordering between them. Users must
know which skill follows which. Most skills are islands — they don't declare predecessors
or successors. There is no way to run "idea to merged PR" as a single command.

## Solution

Two layers:

1. **Skill chain edges** — each SKILL.md frontmatter gets `requires:` and `next:` fields
   declaring predecessor/successor relationships. Advisory metadata consumed by the
   orchestrator.

2. **Named pipelines** — YAML files in `pipelines/` that compose skills into end-to-end
   workflows. A state machine driven by `gm-orchestrator`, not hooks.

## Skill Chain Edges

New optional frontmatter fields in SKILL.md:

```yaml
---
name: "godmode:code-review"
requires: []
next: ["verification-before-completion"]
---
```

### Full Edge Map

```
brainstorm
  next: [writing-plans]

context-map
  next: [writing-plans]

writing-plans
  requires: [context-map]
  next: [task-management]

task-management
  next: [task-driven-development, parallel-agents]

task-driven-development
  next: [code-review]

refactoring
  requires: [testing-philosophy]
  next: [code-review]

code-review
  next: [verification-before-completion]

receiving-review
  next: [verification-before-completion]

verification-before-completion
  next: [cap]

cap
  next: [pr-author]

pr-author
  next: [merge]

merge
  next: [changelog, release-notes]

parallel-agents
  next: [wave-integration]

wave-integration
  next: [code-review]

issue-triage
  next: [task-management]

tackle-issues
  next: [wave-integration]

ci-fix
  next: [cap]

systematic-debugging
  next: [task-driven-development]

dep-audit
  next: [dep-bump]

dep-bump
  next: [cap]

introspection
  next: [doc-maintainer]

doc-maintainer
  next: [cap]

self-reflect
  next: [pattern-learner, mistake-tracker]

health-score
  next: [dead-code, dep-audit]

changelog
  next: [release-notes]
```

### Rules

- `requires` is advisory — the orchestrator warns if skipped, does not block
- `next` is a list — the orchestrator picks the first applicable or runs them in
  parallel if independent
- The graph is a DAG — no cycles
- Skills with no `next` are terminal
- Skills with no `requires` are valid entry points

## Named Pipelines

### Pipeline Definitions

Six pipelines covering all major workflows:

#### 1. `feature` — Idea to Merged PR

```yaml
name: feature
description: Core development loop — idea to merged PR
steps:
  - skill: brainstorm
    optional: true
  - skill: context-map
  - skill: writing-plans
  - skill: task-management
  - skill: task-driven-development
    loop: per-task
  - skill: code-review
  - skill: verification-before-completion
  - skill: cap
  - skill: pr-author
    optional: true
  - skill: merge
    optional: true
entry_points: [brainstorm, task-management]
```

#### 2. `parallel-feature` — Multi-Crate Feature with Parallel Agents

```yaml
name: parallel-feature
description: Fan-out feature development across crates
steps:
  - skill: brainstorm
    optional: true
  - skill: context-map
  - skill: writing-plans
  - skill: task-management
  - skill: parallel-agents
  - skill: wave-integration
  - skill: code-review
  - skill: verification-before-completion
  - skill: cap
  - skill: pr-author
    optional: true
  - skill: merge
    optional: true
entry_points: [brainstorm, task-management]
```

#### 3. `release` — Cut a Release

```yaml
name: release
description: Health check, audit, changelog, release notes, tag
steps:
  - skill: health-score
  - skill: dep-audit
  - skill: code-review
  - skill: verification-before-completion
  - skill: changelog
  - skill: release-notes
  - skill: cap
entry_points: [health-score]
```

#### 4. `maintenance` — Codebase Hygiene

```yaml
name: maintenance
description: Health scorecard followed by targeted cleanup
steps:
  - skill: health-score
  - skill: dead-code
    parallel_with: [dep-audit, introspection, doc-maintainer]
  - skill: code-review
  - skill: cap
entry_points: [health-score]
```

#### 5. `triage` — Issue Backlog to Task Graph

```yaml
name: triage
description: Triage open issues into a prioritized task graph
steps:
  - skill: issue-triage
  - skill: task-management
  - skill: task-driven-development
    loop: per-task
  - skill: code-review
  - skill: verification-before-completion
  - skill: cap
entry_points: [issue-triage]
```

#### 6. `retrospective` — Session Learning

```yaml
name: retrospective
description: End-of-session reflection and knowledge extraction
steps:
  - skill: self-reflect
  - skill: pattern-learner
  - skill: mistake-tracker
  - skill: memory-banking
entry_points: [self-reflect]
```

### Pipeline State

`.ctx/GODMODE.pipeline.yaml` — persists across sessions, gitignored.

```yaml
active: feature
current_step: 4
started_at: "2026-06-02T14:30:00Z"
history:
  - skill: brainstorm
    status: done
    completed_at: "2026-06-02T14:32:00Z"
  - skill: context-map
    status: done
    completed_at: "2026-06-02T14:35:00Z"
  - skill: writing-plans
    status: done
    completed_at: "2026-06-02T14:45:00Z"
  - skill: task-management
    status: running
```

### CLI

```
godmode pipeline list                           # show all pipelines
godmode pipeline show <name>                    # show steps with current position
godmode pipeline start <name> [--from <skill>]  # start and auto-invoke first step
godmode pipeline next                           # advance and auto-invoke next step
godmode pipeline skip                           # advance without invoking
godmode pipeline stop                           # deactivate pipeline, preserve state
godmode pipeline status                         # show active pipeline + position
```

### Execution Model

`gm-orchestrator` owns the pipeline loop:

1. User says "start the feature pipeline" or invokes `godmode pipeline start feature`
2. Orchestrator reads `pipelines/feature.yaml`, writes initial state to
   `.ctx/GODMODE.pipeline.yaml`
3. Orchestrator invokes the first skill (brainstorm) as the current session's active
   skill — not as a subagent
4. When the skill completes, orchestrator calls `godmode pipeline next`
5. Next step is loaded and invoked immediately
6. `loop: per-task` steps repeat for each runnable task in the graph, calling
   `godmode task next` between iterations
7. `parallel_with` steps dispatch via `parallel-agents` pattern
8. `optional: true` steps execute by default — `pipeline skip` bypasses them
9. If the session ends mid-pipeline, state is persisted. Next session's
   `godmode handon` reports the active pipeline and current position.
   `godmode pipeline next` resumes.
10. `pipeline stop` deactivates without clearing history — `pipeline start --from`
    can resume later

### Orchestrator Changes

`gm-orchestrator` agent currently delegates to gm-tasks, gm-plans, and gm-tdd. It
needs to additionally:

- Read pipeline definitions from `pipelines/*.yaml`
- Manage pipeline state in `.ctx/GODMODE.pipeline.yaml`
- Drive the step loop (invoke skill → advance → invoke next)
- Handle `loop: per-task` by iterating over `godmode task next`
- Handle `parallel_with` by grouping steps and dispatching via `parallel-agents`
- Report pipeline progress in `godmode pipeline status`

## Implementation

### Crate: `godmode-core`

New module: `pipeline.rs`

Types:

- `Pipeline` — parsed from YAML: name, description, steps, entry_points
- `PipelineStep` — skill name, optional flag, loop mode, parallel_with
- `PipelineState` — active pipeline name, current step index, history
- `StepStatus` — Pending, Running, Done, Skipped

Functions:

- `load_pipelines(root: &Path) -> Result<Vec<Pipeline>>`
- `load_state(root: &Path) -> Result<Option<PipelineState>>`
- `save_state(root: &Path, state: &PipelineState) -> Result<()>`
- `advance(state: &mut PipelineState, pipeline: &Pipeline) -> Option<&PipelineStep>`
- `current_step(state: &PipelineState, pipeline: &Pipeline) -> Option<&PipelineStep>`

### Crate: `godmode-cli`

New subcommand group: `Pipeline` with variants: List, Show, Start, Next, Skip,
Stop, Status.

### Files

New:

- `crates/godmode-core/src/pipeline.rs`
- `pipelines/feature.yaml`
- `pipelines/parallel-feature.yaml`
- `pipelines/release.yaml`
- `pipelines/maintenance.yaml`
- `pipelines/triage.yaml`
- `pipelines/retrospective.yaml`

Modified:

- `crates/godmode-core/src/lib.rs` — add `pub mod pipeline`
- `crates/godmode-cli/src/main.rs` — add `Pipeline` subcommand group
- All 42 `skills/*/SKILL.md` — add `requires:` and `next:` frontmatter
- `agents/planner-agent.md` — update gm-orchestrator to handle pipeline loop

## Hook Integration

The orchestrator drives the pipeline; hooks observe and annotate. These are
complementary layers, not competing ones.

### Boundary rule

**Hooks never call `godmode pipeline next`.** Only the orchestrator advances the
state machine. Hooks fire within a step and can influence the orchestrator's
decisions, but they don't own the sequence.

### How hooks interact with pipeline steps

| Hook                          | Fires during                       | Effect on pipeline                                                                                                           |
| ----------------------------- | ---------------------------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| `post-bash-auto-block.nu`     | `task-driven-development` step     | Blocks a task on test failure. Orchestrator sees blocked task via `godmode task next` and handles it (retry, skip, or stop). |
| `pre-agent-task-context.nu`   | Any step that dispatches subagents | Injects running task context into agent prompts. No pipeline state change.                                                   |
| `post-write-plan-ingest.nu`   | `writing-plans` step               | Auto-ingests plan markdown into task graph. Orchestrator picks up tasks on next `godmode task next` call.                    |
| `doob-commit-autocomplete.nu` | `cap` step                         | Syncs completed tasks to doob. No pipeline state change.                                                                     |

### Future hook: `post-pipeline-step.nu`

A PostToolUse hook that fires after the orchestrator completes each step. It does
NOT advance the pipeline — it logs the step transition to the session trace for
observability:

```nu
# Append step completion to session trace
let step = $input.skill_name
let pipeline = $input.pipeline_name
godmode session-trace append --event pipeline-step-done --skill $step --pipeline $pipeline
```

This hook is observability-only. The orchestrator remains the sole driver of
`pipeline next`.w

### Out of Scope

- Pipeline-level undo/rollback
- Conditional branching (if/else in pipeline steps)
- Pipeline composition (pipeline calling another pipeline)
- Pipeline templates with variable substitution
- Hooks that advance or alter pipeline state
