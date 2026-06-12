# Crux Types and Traits Reference

## Table of Contents

1. [Core Value Types](#core-value-types)
2. [Step and Trace Types](#step-and-trace-types)
3. [Error Types](#error-types)
4. [Budget](#budget)
5. [Agent Trait and CruxCtx](#agent-trait-and-cruxctx)
6. [Combinators](#combinators)
7. [Delegation and Speculation](#delegation-and-speculation)
8. [Recovery and Hooks](#recovery-and-hooks)
9. [Registry](#registry)
10. [Orchestrator Types](#orchestrator-types)
11. [Proc Macros](#proc-macros)

---

## Core Value Types

### `Crux<T>` (crux-types)

The execution trace fused with a result. Every agent run produces one.

```rust
pub struct Crux<T> {
    pub id: CruxId,
    pub agent: String,
    pub result: Result<T, CruxErr>,
    pub steps: Vec<Step>,
    pub children: Vec<Crux<serde_json::Value>>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub finished_at: Option<chrono::DateTime<chrono::Utc>>,
}
```

Key methods:

- `value() -> Result<&T, &CruxErr>` — access the result
- `into_value() -> Result<T, CruxErr>` — consume and unwrap
- `causal_chain() -> Vec<&Step>` — failed steps leading to error
- `delegations() -> Vec<Delegation>` — parent/child delegation pairs
- `rejected_branches() -> Vec<&Step>` — speculation losers
- `duration_ms() -> Option<u64>` — wall-clock duration
- `succeeded_count() / failed_count()` — step tallies
- `to_snapshot()` — serialize to `Crux<Value>` for replay

### `CruxId` (crux-types)

ULID-based identifier prefixed with `crux_`.

```rust
let id = CruxId::new(); // "crux_01HXYZ..."
```

---

## Step and Trace Types

### `Step` (crux-types)

A recorded unit of work in the trace.

```rust
pub struct Step {
    pub name: String,
    pub kind: StepKind,
    pub status: StepStatus,
    pub confidence: f32,
    pub input_hash: u64,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub finished_at: Option<chrono::DateTime<chrono::Utc>>,
    pub children: Vec<Step>,
}
```

### `StepKind`

```rust
pub enum StepKind {
    Step,           // Normal step
    Delegation,     // Delegated to child agent
    Speculation,    // Speculative branch
    Pipe,           // Sequential pipeline stage
    JoinAll,        // Parallel fan-out arm
}
```

### `StepStatus`

```rust
pub enum StepStatus {
    Ok,        // Completed successfully
    Failed,    // Error occurred
    Rejected,  // Speculation loser
    Skipped,   // Skipped by recovery hook
}
```

---

## Error Types

### `CruxErr` (crux-types)

```rust
pub enum CruxErr {
    StepFailed { step: String, message: String },
    Delegation { from: String, to: String, source: Box<CruxErr> },
    BudgetExceeded { kind: BudgetKind, limit: u64, used: u64 },
    LowConfidence { step: String, score: f32, threshold: f32 },
    ReplayMismatch { expected: String, got: String },
    AllSpeculationsFailed { name: String },
    Custom(String),
}

// Constructors:
CruxErr::step_failed("step_name", "what went wrong")
CruxErr::low_confidence("step_name", 0.3, 0.5)
```

`is_transient()` returns true for errors likely to succeed on retry
(network timeouts, rate limits).

---

## Budget

```rust
pub enum Budget {
    Tokens(u64),
    Calls(u64),
    Duration(Duration),
    CostCents(u64),
    Combined(Vec<Budget>),
}

// Constructors:
Budget::tokens(10000)
Budget::calls(5)
Budget::duration(Duration::from_secs(60))
Budget::cost_cents(100)
Budget::combined(vec![Budget::tokens(10000), Budget::calls(5)])
```

`BudgetTracker` wraps a `Budget` and tracks consumption:

```rust
let mut tracker = BudgetTracker::new(Budget::tokens(1000));
tracker.consume(500);
assert!(!tracker.is_exceeded());
assert_eq!(tracker.remaining(), 500);
```

---

## Agent Trait and CruxCtx

### `Agent` trait (crux-runtime)

```rust
pub trait Agent: Send + Sync {
    type Input: Send;
    type Output: Send + Serialize + DeserializeOwned;

    fn name() -> &'static str;
    async fn run(ctx: &mut CruxCtx, input: Self::Input) -> Result<Self::Output, CruxErr>;
    fn budget() -> Option<Budget> { None }
}
```

### `CruxCtx` (crux-runtime)

The runtime context injected into agent functions. Inside `#[crux::agent]`
bodies, it is available as `x`.

```rust
// Record a step
let result: T = x.step("name", || async { Ok(value) }).await?;

// Record a step with explicit confidence
let result: T = x.step_with_confidence("name", 0.8, || async { Ok(value) }).await?;

// Sequential pipeline
let result: T = x.pipe("name", input, stages).await?;

// Parallel fan-out
let results: Vec<T> = x.join_all("name", arms).await?;

// Confidence routing
let result: T = x.route_on_confidence("name", score, routes).await?;

// Delegation to child agent
let result: T = x.delegate::<AgentType>("name", input).run().await?;

// Speculation (pick best)
let result: T = x.speculate("name", arms).pick_best_by(|r| r.score).await?;

// Speculation (first success)
let result: T = x.speculate("name", arms).first_ok().await?;

// Output propagation (alias namespace for inter-step data)
x.propagate_output("alias", json!({"key": "value"}));
let val = x.read_output("alias"); // Option<Value>

// Checkpointing to registry
x.checkpoint_to(&registry, &task_id).await?;
x.resume_from(&registry, &task_id).await?;

// Mid-run snapshot
let snapshot: Crux<Value> = x.snapshot();
```

---

## Combinators

### `pipe(name, input, stages)`

Chains sequential closures. Each stage receives the previous stage's output.
Records per-stage steps as `name::stage_label`.

### `join_all(name, arms)`

Fans out via `futures::join_all`. All arms execute concurrently. Records
per-arm steps as `name::arm_label`. Fails if any arm fails.

### `route_on_confidence(name, score, routes)`

Dispatches to a handler based on a confidence score. Routes must be
non-overlapping, gap-free, and cover `[0.0, 1.0]`. Range syntax:
`[0.0, 0.5)` (exclusive end) or `[0.8, 1.0]` (inclusive end).

---

## Delegation and Speculation

### `DelegationBuilder`

```rust
x.delegate::<AgentType>("step_name", input)
    .with_budget(Budget::tokens(1000))  // optional per-call budget
    .run()
    .await?
```

Creates a child `CruxCtx`. The child's `Crux<Value>` is appended to
`parent.children`. Failures wrap in `CruxErr::Delegation`.

### `SpeculationBuilder`

```rust
// Pick best by score
x.speculate("name", arms)
    .pick_best_by(|result| result.score)
    .await?

// First success (short-circuit)
x.speculate("name", arms)
    .first_ok()
    .await?
```

Winner is marked `StepStatus::Ok`, losers are `StepStatus::Rejected`.
If all arms fail, returns `CruxErr::AllSpeculationsFailed`.

---

## Recovery and Hooks

### `Recovery<T>` (crux-runtime)

Hook return type for lifecycle callbacks:

```rust
pub enum Recovery<T> {
    Continue,           // Proceed with the error
    Skip,               // Skip this step, use default
    Retry { max: u32 }, // Retry up to N times
    Escalate,           // Bubble up immediately
    Substitute(T),      // Replace with this value
}
```

### `RecoveryKind` (crux-types)

Serializable subset of `Recovery<T>` (no closures):

```rust
pub enum RecoveryKind {
    Continue,
    Skip,
    Retry { max: u32 },
    Escalate,
}
```

### Lifecycle hooks on CruxCtx

```rust
x.on_step_failure(|err| async { Recovery::Substitute(json!(fallback)) });
```

### Governance (crux-runtime)

```rust
pub trait GovernancePolicy: Send + Sync {
    fn evaluate(&self, request: &ApprovalRequest) -> PolicyAction;
}

pub struct ApprovalRequest {
    pub summary: String,
    pub diff_description: String,
    pub risk_level: RiskLevel, // Low | Medium | High | Critical
}

pub enum ApprovalDecision {
    Approved,
    Denied { reason: String },
    Deferred { timeout_seconds: u64 },
}
```

---

## Registry

### `TaskRegistry<B: RegistryBackend>`

Submit/get/update tasks with CAS (compare-and-swap) semantics.

```rust
let registry = TaskRegistry::new(InMemoryBackend::new());
// or: TaskRegistry::new(RedbBackend::open("path")?);

let id = registry.submit("my_task", json!({"input": "data"})).await?;
let task = registry.get(&id).await?;
registry.update_status(&id, TaskStatus::Done).await?;
```

### `RegistryBackend` trait

Persistence port. Two adapters: `InMemoryBackend` (default) and
`RedbBackend` (behind `redb` feature flag).

---

## Orchestrator Types

### `HarnessProfile` (crux-runtime)

Resource spec for a container/process harness (image, env, limits).

### `ResourceHints`

Advisory scheduling metadata attached to a profile.

### `HarnessDiff`

Incremental description of profile changes, emitted by `EvolutionPlanner`.

### `EvolutionOutcome`

Result of applying a diff: `Accepted`, `Rejected`, or `RequiresApproval`.

### `SafetyPolicy` trait

Port for diff approval logic. Returns `Approved` / `Rejected` /
`RequiresApproval`.

### `ApprovalGate` trait

Hook called when `SafetyPolicy` returns `RequiresApproval`. Adapters:
`AutoApproveGate`, `TerminalApprovalGate`.

---

## Proc Macros

### `#[crux::agent]`

Applied to `async fn name(params) -> Crux<T>`. Generates:

1. Inner function with `CruxCtx` injected as `x`
2. Public wrapper that creates `CruxCtx` and calls `finalize()`
3. `NameAgent` struct implementing the `Agent` trait

```rust
#[crux::agent]
async fn my_agent(input: String) -> Crux<String> {
    let val: String = x.step("process", || async { Ok(input.to_uppercase()) }).await?;
    Ok(val)
}

// Generated: MyAgentAgent struct, my_agent() wrapper function
let crux = my_agent("hello".to_string()).await;
```

Optional attributes:

- `#[crux::agent(replay = "lenient")]` — lenient replay mode
- `#[crux::agent(registry = "task_kind")]` — generates `run_registered()`

### `#[crux::harness]`

Marks a struct as a managed container/process harness. The struct must have
an `image: String` field; additional fields map to `HarnessProfile`.

### `#[crux::evolve]`

Applied to `async fn f(metrics: RunMetrics) -> Crux<EvolutionOutcome>`.
Injects `planner` (`EvolutionPlanner`) and `x` (`CruxCtx`) into the body.
