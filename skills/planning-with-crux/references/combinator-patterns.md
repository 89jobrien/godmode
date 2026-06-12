# Combinator Patterns Reference

## Table of Contents

1. [Single Step](#single-step)
2. [Sequential Pipe](#sequential-pipe)
3. [Parallel Fan-Out](#parallel-fan-out)
4. [Speculation: First OK](#speculation-first-ok)
5. [Speculation: Pick Best](#speculation-pick-best)
6. [Confidence Routing](#confidence-routing)
7. [Delegation](#delegation)
8. [Composition Patterns](#composition-patterns)

---

## Single Step

The simplest unit. One handler invocation.

```yaml
- step: read_config
  handler: fs::read
  args:
    path: "config.toml"
```

If `handler:` is omitted, the step name is used as the handler name.

---

## Sequential Pipe

Stages execute in order. Each stage receives the previous stage's output.
The pipe's output is the last stage's output.

```yaml
- pipe: transform
  stages:
    - step: parse
      handler: text::parse_jsonl
    - step: filter
      handler: json::filter_nonempty
      args:
        field: "status"
    - step: group
      handler: json::group_by
      args:
        key: "category"
```

In traces, steps appear as `transform::parse`, `transform::filter`,
`transform::group`.

**When to use**: Data transformation chains, multi-stage processing where
each stage depends on the previous one.

---

## Parallel Fan-Out

Arms execute concurrently via `futures::join_all`. All arms receive the same
input. Output is a JSON array of all arm outputs.

```yaml
- join_all: gather
  arms:
    - step: git_status
      handler: git::status
    - step: staged_files
      handler: git::staged_files
    - step: recent_commits
      handler: git::log
      args:
        count: 5
```

**Fails if any arm fails.** Design arms to be independently recoverable
or use `shell::exec` (which doesn't fail on non-zero exit) for
best-effort arms.

**When to use**: Independent data gathering, parallel checks, concurrent
builds for different targets.

---

## Speculation: First OK

Arms execute in order. Returns the first successful result. Remaining arms
are not executed (short-circuit).

```yaml
- speculate: find_config
  mode: first_ok
  arms:
    - step: local
      handler: fs::read
      args:
        path: ".config.toml"
    - step: home
      handler: fs::read
      args:
        path: "~/.config/tool/config.toml"
    - step: default
      handler: ctrl::noop
      # Fallback: pass input through as default config
```

**When to use**: Fallback chains — try the preferred option first, fall
back to alternatives.

---

## Speculation: Pick Best

All arms execute concurrently. Each arm's output must include a `score`
field (numeric). The arm with the highest score wins; losers are marked
`Rejected` in the trace.

```yaml
- speculate: choose_strategy
  mode: pick_best
  arms:
    - step: fast_approach
      handler: shell::capture
      args:
        cmd: "time_and_score fast_method"
    - step: thorough_approach
      handler: shell::capture
      args:
        cmd: "time_and_score thorough_method"
```

**When to use**: When you have multiple strategies and want to
empirically pick the best one.

---

## Confidence Routing

Dispatches to exactly one handler based on a confidence score. Routes
must be non-overlapping, gap-free, and collectively cover `[0.0, 1.0]`.

```yaml
- route_on_confidence: decide
  value: "{{ steps.score_step.confidence }}"
  routes:
    - range: "[0.0, 0.4)"
      label: escalate
      handler: ctrl::log
    - range: "[0.4, 0.8)"
      label: auto_fix
      handler: shell::capture
      args:
        cmd: "cargo fix --allow-dirty"
    - range: "[0.8, 1.0]"
      label: approve
      handler: ctrl::log
```

**Range syntax**: `[lo, hi)` = inclusive start, exclusive end.
`[lo, hi]` = both inclusive. The last range should usually be inclusive
on both ends to catch 1.0.

**When to use**: After a scoring/classification step, to take different
actions at different confidence levels. Classic pattern: low = manual,
medium = auto-fix, high = approve.

---

## Delegation

Hand off to a named sub-agent. The sub-agent runs with its own `CruxCtx`
and the result is merged back into the parent trace.

```yaml
- delegate: analyzer_agent
  name: analyze
  budget: { tokens: 5000 }
```

In Rust:

```rust
let result = x.delegate::<AnalyzerAgent>("analyze", input)
    .with_budget(Budget::tokens(5000))
    .run()
    .await?;
```

**When to use**: When a portion of the workflow is complex enough to
warrant its own agent with its own trace, or when you want budget
isolation.

---

## Composition Patterns

### Gather → Process → Act

The most common pipeline shape. Parallel data gathering, sequential
processing, confidence-based action.

```yaml
steps:
  - join_all: gather
    arms:
      - step: source_a
        handler: shell::capture
        args: { cmd: "fetch_a" }
      - step: source_b
        handler: shell::capture
        args: { cmd: "fetch_b" }

  - pipe: process
    stages:
      - step: merge
        handler: json::merge
      - step: classify
        handler: ci::classify_severity
      - step: score
        handler: ci::score_fixability

  - route_on_confidence: act
    value: "{{ steps.process.confidence }}"
    routes:
      - range: "[0.0, 0.5)"
        label: manual
        handler: ctrl::log
      - range: "[0.5, 1.0]"
        label: auto
        handler: shell::capture
        args: { cmd: "auto_fix" }
```

### Speculate → Validate

Try multiple approaches, pick the best, then validate the winner.

```yaml
steps:
  - speculate: attempt
    mode: pick_best
    arms:
      - step: approach_a
        handler: llm::invoke
      - step: approach_b
        handler: llm::invoke

  - pipe: validate
    stages:
      - step: check_syntax
        handler: shell::capture
        args: { cmd: "lint" }
      - step: check_tests
        handler: shell::capture
        args: { cmd: "cargo test" }
```

### Gate → Work → Gate

Validation bookends around the main work.

```yaml
steps:
  - pipe: pre_gate
    stages:
      - step: clean_tree
        handler: git::status
      - step: run_ci
        handler: shell::capture
        args: { cmd: "just ci" }

  - join_all: build
    arms:
      - step: target_a
        handler: shell::capture
        args: { cmd: "cross build --target x86_64-unknown-linux-musl" }
      - step: target_b
        handler: shell::capture
        args: { cmd: "cargo build --target aarch64-apple-darwin" }

  - pipe: post_gate
    stages:
      - step: verify_binaries
        handler: shell::capture
        args: { cmd: "file target/*/release/*" }
      - step: tag
        handler: shell::capture
        args: { cmd: "git tag -a v1.0.0 -m release" }
```
