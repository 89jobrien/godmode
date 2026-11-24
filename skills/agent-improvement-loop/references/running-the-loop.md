# Running the loop

Read this before starting a run of the improvement loop, to choose a full or partial run and set the invocation flags.

## Running the Loop

### Full loop (interactive)

Work through phases 1–7 in order. Present findings at each phase boundary and wait for
confirmation before proceeding to the next phase.

### Partial loop

- **Diagnose only**: run phases 1–4, emit `diagnosis.json`, stop
- **Eval-first**: skip phase 1, start from existing `feedback.json` at phase 3
- **Harness-only**: given a `diagnosis.json`, jump to phase 5 and produce the handoff

### Loop invocation

```text
/agent-improvement-loop [--agent <name>] [--since <days>] [--phase <N>] [--dry-run]
```

| Flag        | Default         | Effect                                   |
| ----------- | --------------- | ---------------------------------------- |
| `--agent`   | current project | Which agent's traces to analyze          |
| `--since`   | 30              | Days of trace history to scan            |
| `--phase`   | 1               | Start from this phase                    |
| `--dry-run` | false           | Run through phases without writing files |
