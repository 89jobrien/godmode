# pipelines/

Declarative skill pipelines — ordered sequences of skills with entry points and loop
annotations. Consumed by the pipeline runner to orchestrate multi-step workflows.

## Format

```yaml
name: feature
description: Core development loop — idea to merged PR
steps:
  - skill: brainstorm
    optional: true
  - skill: writing-plans
  - skill: ingest
  - skill: task-management
  - skill: task-driven-development
    loop: per-task # repeat this step for each task in the graph
    parallel_with: [code-review]
  - skill: cap
    optional: true
entry_points: [brainstorm, task-management]
```

Fields:

- `optional: true` — step is skipped if the user declines
- `loop: per-task` — step repeats for each pending task
- `parallel_with` — companion skills available for concurrent orchestration; the current
  headless runner records this metadata but does not execute companion skills automatically
- `entry_points` — skills where the pipeline can start mid-flow

## Pipelines

| Pipeline                | Description                                          |
| ----------------------- | ---------------------------------------------------- |
| `feature.yaml`          | Core development loop — idea to merged PR            |
| `release.yaml`          | Release pipeline — readiness check to published tag  |
| `maintenance.yaml`      | Dependency updates and dead code cleanup             |
| `lifecycle.yaml`        | Full session lifecycle from handon to handoff        |
| `parallel-feature.yaml` | Feature development with parallel agent dispatch     |
| `retrospective.yaml`    | Session retrospective and memory bank update         |
| `triage.yaml`           | Issue triage and backlog grooming                    |
| `aichat-system.yaml`    | aichat system prompt generation and installation     |
| `coursers-rules.yaml`   | Coursers rule lifecycle — discover to installed rule |

## Relationship to Commands

Claude `/gm:*` and OpenCode `/gm-*` workflow commands implement the same sequences.
Pipelines are the target-neutral machine-readable equivalent, used by the pipeline runner
(`godmode pipeline next`, `post-pipeline-step.nu`) for automated step advancement.
