# Pipelines

Read this before starting, advancing, or running a pipeline.

```text
godmode pipeline list                           # show all pipelines
godmode pipeline show <name>                    # show steps with current position
godmode pipeline start <name> [--from <skill>]  # start and auto-invoke first step
godmode pipeline next                           # advance and invoke next step
godmode pipeline skip                           # advance without invoking
godmode pipeline stop                           # deactivate pipeline, preserve state
godmode pipeline status                         # show active pipeline + position
godmode pipeline run <name> [--from <skill>] [--fail-fast]  # headless: walk task graph, execute run: fields
```

Six pipelines are defined in `pipelines/`:

- `feature` — idea to merged PR
- `parallel-feature` — fan-out across crates
- `release` — health check, audit, changelog, tag
- `maintenance` — health scorecard + targeted cleanup
- `triage` — issue backlog to task graph
- `retrospective` — session reflection and learning

Pipeline state persists in `.ctx/godmode/pipeline.yaml` (gitignored).
`task clear` requires `--done` (completed only) or `--all`.
