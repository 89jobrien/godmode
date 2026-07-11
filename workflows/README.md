# workflows/

Multi-agent workflow scripts for the Claude Code Workflow tool. Each subdirectory is a
named workflow that can be invoked via the `Workflow` tool.

## Workflows

| Workflow      | Description                                                                  |
| ------------- | ---------------------------------------------------------------------------- |
| `brainstorm/` | Multi-agent design exploration — advocate, skeptic, alternative, synthesizer |
| `tdd/`        | Parallel TDD implementation across workspace crates                          |

## Structure

Each workflow directory contains:

- `design.yaml` or `cycle.yaml` — workflow script invoked by the Workflow tool
- Supporting agent definitions or config as needed

## Relationship to Commands and Pipelines

- **Commands** (`commands/gm/`) — Claude follows instructions manually via the Skill tool
- **Pipelines** (`pipelines/`) — declarative step sequences for the pipeline runner
- **Workflows** (`workflows/`) — multi-agent fan-out scripts run by the Workflow tool

Use workflows when genuinely independent parallel subagents are needed (e.g. parallel
crate implementation). Use commands for sequential human-in-the-loop workflows.
