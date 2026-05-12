# Testing Primitives (godmode-core, `testing` feature)

Reusable test infrastructure primitives for any hexagonal Rust workspace.

Enable with: `godmode-core = { features = ["testing"] }`

## ConformanceSuite

Write test cases once against a port trait, run against every adapter.
Enforces LSP — no adapter can silently break a contract.

```rust
use godmode_core::testing::conformance::{ConformanceCase, ConformanceSuite};

struct MustReturnNonEmpty;
impl ConformanceCase<dyn MyPort> for MustReturnNonEmpty {
    fn name(&self) -> &str { "returns_non_empty" }
    fn run(&self, subject: &dyn MyPort) -> Result<(), String> {
        if subject.fetch().is_empty() { Err("empty".into()) } else { Ok(()) }
    }
}

fn suite() -> ConformanceSuite<dyn MyPort> {
    ConformanceSuite::new().case(Box::new(MustReturnNonEmpty))
}

#[test]
fn real_adapter_conforms() { suite().assert_all(&RealAdapter); }

#[test]
fn mock_adapter_conforms() { suite().assert_all(&MockAdapter); }
```

## assert_implements!

Zero-cost compile-time trait-bound verification:

```rust
use godmode_core::testing::audit::assert_implements;
assert_implements!(MyConfig: Debug + Clone + Send + Sync);
```

## DepAudit

Guard lightweight crates against accidental heavy deps:

```rust
use godmode_core::testing::audit::DepAudit;
DepAudit::new("Cargo.toml")
    .allow("serde").allow("anyhow")
    .assert_no_unlisted();
```

## SnapshotAudit

Lightweight golden-file testing (alternative to insta):

```rust
use godmode_core::testing::audit::SnapshotAudit;
let mut audit = SnapshotAudit::new("tests/snapshots/api_fields.txt");
audit.record("field_a", "present");
audit.assert_snapshot().unwrap();
```

## TestContext (RAII env guard)

```rust
use godmode_core::testing::env::TestContext;
let _ctx = TestContext::builder()
    .env("DATABASE_URL", "postgres://test")
    .build();
// restored on drop
```

## PropConfig + assert_round_trip

```rust
use godmode_core::testing::prop::{PropConfig, assert_round_trip};
assert_round_trip(any::<MyType>());  // serde JSON roundtrip
```

## deterministic_seed

```rust
use godmode_core::testing::seed::deterministic_seed;
let seed = deterministic_seed("my_test_name"); // stable u64
```

## Patterns (not in crate, adopt manually)

- **AST guardrails**: Parse source with `syn`, assert structural invariants
  (no type leakage across crate boundaries). Template in
  `testing-philosophy/helpers/`.
- **`never_connect_pool()`**: Use `connect_lazy` + short acquire_timeout to
  test SQL construction without a running database.
- **RunStatus FSM**: `is_terminal()` guard + `Err(Protocol)` on invalid
  state transitions at the DB boundary.
- **Partial index**: `CREATE INDEX ... WHERE status = 'active'` for queue
  tables with time-based dispatch.
