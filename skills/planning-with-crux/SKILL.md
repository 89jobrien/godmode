---
name: "planning-with-crux"
description: >
  Design and author agentic workflows using the crux DSL — both YAML pipeline
  files (.crux) and Rust macro-based agents. Use when writing a .crux pipeline,
  designing an agent workflow, choosing between crux combinators (pipe, join_all,
  speculate, route_on_confidence, delegate), or planning how to decompose a task
  into crux steps. Triggers on "write a pipeline", "crux pipeline", ".crux file",
  "agentic workflow", "agent plan", "step handler", "which combinator", or any
  request involving pipeline design, step composition, or handler selection.
  Also use when reviewing or debugging existing pipelines.
---

# Planning with Crux

Crux makes agentic control flow explicit. Every step, branch, delegation, and
failure is a first-class value in the trace. This skill helps you design
workflows that use these primitives effectively.

## Two Modes of Authoring

1. **YAML pipelines** (`.crux` files) — declarative, no Rust compilation needed.
   Run with `crux run pipeline.crux [input.json]`. Good for orchestration tasks
   that compose built-in handlers.

2. **Rust agents** (`#[crux::agent]`) — full type safety, custom logic, async.
   Compile into binaries. Good for complex domain logic that needs Rust's type
   system and error handling.

Choose YAML when the workflow is a composition of existing handlers (shell
commands, file ops, git, JSON transforms, LLM calls). Choose Rust when you
need custom types, complex branching, or the workflow will be a library.

## Pipeline Anatomy

```yaml
pipeline: my_workflow # Required: name (appears in trace)
budget: { calls: 10 } # Optional: tokens, calls, duration_ms, cost_cents
steps: # Required: list of step definitions
  - step: first_step
    handler: shell::capture
    args:
      cmd: "echo hello"
```

Every pipeline produces a `Crux<Value>` trace. The last step's output is the
pipeline result.

## Choosing a Combinator

| Pattern                   | Combinator             | When to use                            |
| ------------------------- | ---------------------- | -------------------------------------- |
| Do A, then B, then C      | `pipe:`                | Sequential stages, each feeds next     |
| Do A, B, C simultaneously | `join_all:`            | Independent work, merge results        |
| Try multiple approaches   | `speculate: first_ok`  | Fallback chain, first success wins     |
| Pick the best attempt     | `speculate: pick_best` | Race approaches, score and select      |
| Branch on confidence      | `route_on_confidence:` | Different actions for different scores |
| Hand off to sub-agent     | `delegate:`            | Encapsulated child agent               |
| Single action             | `step:`                | One handler invocation                 |

Read `references/combinator-patterns.md` for worked examples of each pattern
and guidance on composing them together.

## Handler Selection

Read `references/handler-catalog.md` for the complete handler reference
organized by category (shell, fs, git, json, text, llm, ci, review, etc.).

Key principles:

- **`shell::capture`** fails on non-zero exit; **`shell::exec`** ignores it.
  Use `capture` when the command must succeed. Use `exec` for best-effort.
- **`handler()` vs `handler_value()`**: If a handler needs to emit confidence
  for `route_on_confidence`, register with `handler()` and return
  `HandlerOutput::with_confidence(value, score)`. If it just returns data,
  use `handler_value()`.
- **Template expressions** `{{ steps.X.output.field }}` and `{{ input.field }}`
  expand in `args:` values. Use these to pass data between steps.

## Budget Design

Set budgets to prevent runaway execution:

```yaml
budget: { calls: 10 }             # Max 10 handler invocations
budget: { tokens: 15000 }         # Max 15k tokens (LLM steps)
budget: { duration_ms: 300000 }   # Max 5 minutes wall clock
budget: { cost_cents: 50 }        # Max $0.50
```

For Rust agents, budgets can be scoped per delegation:

```rust
x.delegate::<SubAgent>("task", input)
    .with_budget(Budget::tokens(5000))
    .run()
    .await?
```

## Pipeline Design Process

1. **Name the outcome** — what does the pipeline produce?
2. **List the steps** — what information or actions are needed?
3. **Identify dependencies** — which steps need outputs from other steps?
4. **Choose combinators** — sequential (pipe), parallel (join_all), or branching?
5. **Set the budget** — what's the maximum acceptable cost/time?
6. **Add confidence routing** — should different confidence levels trigger
   different actions?

## Rust Agent Patterns

See `references/rust-agent-patterns.md` for the `#[crux::agent]` macro
patterns, including steps, delegation, speculation, lifecycle hooks, replay,
and task registry integration.

## Common Mistakes

- **Forgetting `handler:` on a step** — the step name is used as handler name
  if `handler:` is omitted. This works if your handler is named the same as
  your step, but fails silently if not.
- **Non-covering confidence routes** — ranges must cover `[0.0, 1.0]` with no
  gaps. Use `[0.0, 0.5)`, `[0.5, 0.8)`, `[0.8, 1.0]` — note the inclusive
  end on the last range.
- **Using `handler_value()` with `route_on_confidence`** — `handler_value()`
  handlers don't emit confidence. Use `handler()` and return
  `HandlerOutput::with_confidence()`.
- **`speculate: pick_best` without `score`** — arms must include a `score`
  field in their output JSON. Arms without `score` all tie at 0.0.
