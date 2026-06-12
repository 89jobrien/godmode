# Rust Agent Patterns

Patterns for writing agents with `#[crux::agent]` and the crux runtime API.

## Table of Contents

1. [Basic Agent](#basic-agent)
2. [Steps with Confidence](#steps-with-confidence)
3. [Sequential Pipe](#sequential-pipe)
4. [Parallel Fan-Out](#parallel-fan-out)
5. [Delegation](#delegation)
6. [Speculation](#speculation)
7. [Confidence Routing](#confidence-routing)
8. [Lifecycle Hooks](#lifecycle-hooks)
9. [Replay](#replay)
10. [Task Registry Integration](#task-registry-integration)

---

## Basic Agent

```rust
use crux::prelude::*;

#[crux::agent]
async fn greet(name: String) -> Crux<String> {
    let upper: String = x
        .step("uppercase", || {
            let n = name.clone();
            async move { Ok(n.to_uppercase()) }
        })
        .await?;

    Ok(format!("Hello, {upper}!"))
}

// Usage:
let crux = greet("world".to_string()).await;
assert_eq!(crux.value().unwrap(), "Hello, WORLD!");
assert_eq!(crux.steps.len(), 1);
assert_eq!(crux.agent, "greet");
```

The macro generates:

- `GreetAgent` struct implementing `Agent` trait
- `greet()` wrapper function returning `Crux<String>`
- `x: CruxCtx` injected into the function body

---

## Steps with Confidence

```rust
#[crux::agent]
async fn classify(text: String) -> Crux<String> {
    let label: String = x
        .step_with_confidence("classify", 0.85, || async {
            Ok("positive".to_string())
        })
        .await?;
    Ok(label)
}
// crux.steps[0].confidence == 0.85
```

---

## Sequential Pipe

```rust
#[crux::agent]
async fn transform(input: String) -> Crux<String> {
    let result: String = x
        .pipe(
            "transform",
            input,
            vec![
                ("upper", stage(|s| Box::pin(async move { Ok(s.to_uppercase()) }))),
                ("trim",  stage(|s| Box::pin(async move { Ok(s.trim().to_string()) }))),
                ("bang",  stage(|s| Box::pin(async move { Ok(format!("{s}!")) }))),
            ],
        )
        .await?;
    Ok(result)
}
// Steps recorded: transform::upper, transform::trim, transform::bang
```

---

## Parallel Fan-Out

```rust
#[crux::agent]
async fn gather(_: ()) -> Crux<Vec<i32>> {
    let results: Vec<i32> = x
        .join_all(
            "fetch",
            vec![
                ("a", Box::pin(async { Ok(10_i32) })),
                ("b", Box::pin(async { Ok(20_i32) })),
                ("c", Box::pin(async { Ok(30_i32) })),
            ],
        )
        .await?;
    Ok(results) // [10, 20, 30]
}
```

---

## Delegation

```rust
#[crux::agent]
async fn parent(n: i32) -> Crux<i32> {
    // Basic delegation
    let doubled = x.delegate::<DoublerAgent>("double", n)
        .run()
        .await?;

    // With budget
    let tripled = x.delegate::<TriplerAgent>("triple", n)
        .with_budget(Budget::tokens(1000))
        .run()
        .await?;

    Ok(doubled + tripled)
}
```

The child agent's `Crux<Value>` is appended to `parent.children`.
Failures wrap in `CruxErr::Delegation { from, to, source }`.

---

## Speculation

### Pick Best

```rust
let result: Scored = x
    .speculate(
        "choose",
        vec![
            ("fast",     Box::pin(async { Ok(Scored { value: "fast", score: 0.6 }) })),
            ("thorough", Box::pin(async { Ok(Scored { value: "thorough", score: 0.9 }) })),
        ],
    )
    .pick_best_by(|r| r.score)
    .await?;
// result.value == "thorough" (highest score)
// Winner: StepStatus::Ok, losers: StepStatus::Rejected
```

### First OK (short-circuit)

```rust
let result: i32 = x
    .speculate(
        "fallback",
        vec![
            ("primary",   Box::pin(async { Err(CruxErr::step_failed("primary", "down")) })),
            ("secondary", Box::pin(async { Ok(42) })),
            ("tertiary",  Box::pin(async { Ok(99) })), // never executes
        ],
    )
    .first_ok()
    .await?;
// result == 42
```

---

## Confidence Routing

```rust
let label: String = x
    .route_on_confidence(
        "classify",
        confidence_score,
        vec![
            (ConfidenceRange::exclusive(0.0, 0.5), "low",
             Box::pin(async { Ok("needs review".to_string()) })),
            (ConfidenceRange::exclusive(0.5, 0.8), "medium",
             Box::pin(async { Ok("auto-fixable".to_string()) })),
            (ConfidenceRange::inclusive(0.8, 1.0), "high",
             Box::pin(async { Ok("approved".to_string()) })),
        ],
    )
    .await?;
```

---

## Lifecycle Hooks

```rust
#[crux::agent]
async fn resilient() -> Crux<i32> {
    // Substitute a fallback value on step failure
    x.on_step_failure(|_err| async {
        Recovery::Substitute(serde_json::json!(0))
    });

    let val: i32 = x
        .step("risky", || async {
            Err(CruxErr::step_failed("risky", "network timeout"))
        })
        .await?;

    Ok(val) // Returns 0 via substitution
}
```

Recovery variants: `Continue`, `Skip`, `Retry { max }`, `Escalate`,
`Substitute(T)`.

---

## Replay

```rust
// First run
let first = my_agent("input".to_string()).await;
let snapshot = first.to_snapshot()?;

// Replay from prior trace
let mut ctx = CruxCtx::new("my_agent");
ctx.set_replay_mode(ReplayMode::Lenient);
ctx.replay_from(&snapshot);
let result = MyAgentAgent::run(&mut ctx, "input".to_string()).await;
```

Use `#[crux::agent(replay = "lenient")]` to set lenient mode by default.

---

## Task Registry Integration

```rust
#[crux::agent(registry = "process")]
async fn registered(input: String) -> Crux<String> {
    let val: String = x
        .step("transform", || async { Ok(input.to_uppercase()) })
        .await?;
    Ok(val)
}

// Creates a task, runs agent, marks task done/failed
let registry = TaskRegistry::new(InMemoryBackend::new());
let (crux, task_id) = RegisteredAgent::run_registered(&registry, "hello".to_string()).await;
```
