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
  - skill: task-driven-development
    loop: per-task # repeat this step for each task in the graph
  - skill: cap
    optional: true
entry_points: [brainstorm, task-management]
```

Fields:

- `optional: true` — step is skipped if the user declines
- `loop: per-task` — step repeats for each pending task
- `entry_points` — skills where the pipeline can start mid-flow

## Pipelines

| Pipeline                | Description                                         |
| ----------------------- | --------------------------------------------------- |
| `feature.yaml`          | Core development loop — idea to merged PR           |
| `release.yaml`          | Release pipeline — readiness check to published tag |
| `maintenance.yaml`      | Dependency updates and dead code cleanup            |
| `lifecycle.yaml`        | Full session lifecycle from handon to handoff       |
| `parallel-feature.yaml` | Feature development with parallel agent dispatch    |
| `retrospective.yaml`    | Session retrospective and memory bank update        |
| `triage.yaml`           | Issue triage and backlog grooming                   |
| `aichat-system.yaml`    | aichat system prompt generation and installation    |

## Relationship to Commands

`/gm:*` workflow commands implement the same sequences inline as Claude instructions.
Pipelines are the machine-readable equivalent — used by the pipeline runner
(`godmode pipeline next`, `post-pipeline-step.nu`) for automated step advancement.
