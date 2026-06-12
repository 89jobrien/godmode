# Crux Architecture Reference

## Table of Contents

1. [Hexagonal Design](#hexagonal-design)
2. [SOLID Decomposition](#solid-decomposition)
3. [Replay System](#replay-system)
4. [Context Abstraction](#context-abstraction)
5. [Pipeline Execution](#pipeline-execution)

---

## Hexagonal Design

Crux follows ports-and-adapters (hexagonal) architecture. Domain logic lives
in `crux-runtime`; external concerns are injected via trait ports.

### Persistence Port

```
RegistryBackend (trait)
  ├── InMemoryBackend (default, always available)
  └── RedbBackend (behind `redb` feature flag)
```

### Safety Port

```
SafetyPolicy (trait) → Approved | Rejected | RequiresApproval
  └── ApprovalGate (trait) — called when RequiresApproval
        ├── AutoApproveGate
        └── TerminalApprovalGate
```

### Handler Port (crux-script)

```
HandlerRegistry
  ├── handler(name, fn) → HandlerOutput (with confidence)
  └── handler_value(name, fn) → Value (auto-wrapped, confidence = 1.0)
```

---

## SOLID Decomposition

`CruxCtx` delegates to independently testable collaborators:

| Collaborator   | File          | Responsibility                                  |
| -------------- | ------------- | ----------------------------------------------- |
| `HookRegistry` | `hooks.rs`    | Lifecycle hook dispatch (on_step_failure, etc.) |
| `StepRecorder` | `recorder.rs` | Appends steps to the trace                      |
| `ReplayCache`  | `replay.rs`   | Step output cache with strict/lenient modes     |

Each collaborator can be tested in isolation without constructing a full
`CruxCtx`.

---

## Replay System

Steps are matched by name + ordinal hash (`hash_step_identity`).

### Strict mode (default)

Fails on mismatch. The step sequence must exactly match the prior trace.

### Lenient mode

Forward name scan when ordinal doesn't match. This is the designed recovery
path for traces where step order has shifted — it is not a fallback, it is
the primary mechanism for resilient replay.

### Usage

```rust
let snapshot = prior_crux.to_snapshot()?;

let mut ctx = CruxCtx::new("agent_name");
ctx.set_replay_mode(ReplayMode::Lenient);
ctx.replay_from(&snapshot);

let result = AgentType::run(&mut ctx, input).await;
```

---

## Context Abstraction

The `Context` trait (`context.rs`) is the DIP abstraction over `CruxCtx`
for testability. `Agent::run` takes `&mut CruxCtx` directly — use the
`Context` trait boundary to inject mocks in unit tests.

---

## Pipeline Execution

`crux-script` interprets `.crux` YAML files against `CruxCtx` +
`HandlerRegistry`.

```
PipelineDef (schema.rs)
  → Runner::run(pipeline, input) (runner.rs)
    → CruxCtx created with pipeline name
    → execute_steps() iterates StepDef variants
    → Each step variant maps to CruxCtx combinator:
        StepDef::Step      → ctx.step()
        StepDef::Pipe      → ctx.pipe()
        StepDef::JoinAll   → ctx.join_all()
        StepDef::Speculate → ctx.speculate().pick_best_by() / .first_ok()
        StepDef::RouteOnConfidence → ctx.route_on_confidence()
        StepDef::Delegate  → ctx.step() (agent lookup)
    → ctx.finalize(result) produces Crux<Value>
```

### Expression Context

Template strings in step args (`{{ steps.X.output.field }}`,
`{{ input.field }}`) are expanded via `ExprContext` before handler
invocation. Expansion errors are silently ignored — the original string
is preserved.

### Handler Output

```rust
pub struct HandlerOutput {
    pub value: Value,
    pub confidence: Option<f32>,
}
```

- `handler()` registered handlers return `HandlerOutput` directly
- `handler_value()` registered handlers return plain `Value` (confidence
  defaults to `None`, not 1.0 — `route_on_confidence` can distinguish)
