# rustqual.toml — Workspace Configuration Examples

Annotated configs for common workspace shapes. Place `rustqual.toml` at the repo
root for workspace-level scans, or inside a crate directory for per-crate overrides.

---

## Shape 1: Single lib crate consumed by one binary

```
my-project/
├── Cargo.toml          # workspace root
├── rustqual.toml       # workspace-level config (this file)
├── crates/
│   ├── my-lib/         # library crate — pub API consumed by my-cli
│   └── my-cli/         # binary crate — dispatches to my-lib
```

```toml
# rustqual.toml (workspace root)

# CLI dispatch and main are pure wiring — no logic to unit test.
ignore_functions = ["main", "run", "dispatch", "dispatch_*", "handle_*"]

# All DEAD_CODE findings will be cross-crate false positives (my-lib pub API
# used only by my-cli). Disable rather than suppress inline.
[duplicates]
detect_dead_code = false

[complexity]
max_cognitive = 18
max_cyclomatic = 11
max_function_lines = 68

[srp]
max_parameters = 5
file_length_baseline = 300
file_length_ceiling = 800
lcom4_threshold = 2

[weights]
iosp         = 0.22
complexity   = 0.18
dry          = 0.13
srp          = 0.18
coupling     = 0.09
test_quality = 0.10
architecture = 0.10
```

---

## Shape 2: Multi-crate workspace (core + adapters + CLI)

```
my-project/
├── Cargo.toml
├── rustqual.toml           # workspace root — permissive dead_code setting
├── crates/
│   ├── core/               # domain logic — heavily tested
│   │   └── rustqual.toml   # stricter per-crate override
│   ├── adapter-http/       # HTTP adapter — pub API used by runtime
│   ├── adapter-db/         # DB adapter — pub API used by runtime
│   └── runtime/            # binary — wires adapters to core
```

```toml
# rustqual.toml (workspace root)
ignore_functions = ["main", "run", "dispatch_*"]

[duplicates]
# Adapters export pub API used across crates — all DEAD_CODE is structural.
detect_dead_code = false

[complexity]
max_cognitive = 20     # slightly relaxed for adapter glue code
max_cyclomatic = 12
max_function_lines = 80

[srp]
max_parameters = 6     # adapter constructors often have more params
file_length_baseline = 300
file_length_ceiling = 900
lcom4_threshold = 2

[weights]
iosp         = 0.22
complexity   = 0.18
dry          = 0.13
srp          = 0.18
coupling     = 0.09
test_quality = 0.10
architecture = 0.10
```

```toml
# crates/core/rustqual.toml (per-crate override — stricter)
# Pure domain logic: no I/O boundaries, no dispatch wiring.
# Every function should be unit-testable. No exceptions.
ignore_functions = []

[duplicates]
detect_dead_code = true   # genuine dead code is a defect here

[complexity]
max_cognitive = 15        # tighter — domain logic should be simple
max_cyclomatic = 9
max_function_lines = 50

[srp]
max_parameters = 4
file_length_baseline = 200
file_length_ceiling = 600
lcom4_threshold = 2

[weights]
iosp         = 0.25       # IOSP matters most in domain logic
complexity   = 0.20
dry          = 0.15
srp          = 0.20
coupling     = 0.05
test_quality = 0.10
architecture = 0.05
```

---

## Shape 3: Workspace with cfg-gated code (Kani proofs, testutil)

```
my-project/
├── Cargo.toml
├── rustqual.toml
├── crates/
│   └── core/
│       └── src/
│           ├── lib.rs
│           ├── kani_proofs.rs    # #[cfg(kani)] — invisible to rustqual
│           └── testutil.rs       # #[cfg(any(test, feature="testutil"))]
```

```toml
# rustqual.toml
ignore_functions = ["main"]

# Exclude cfg-gated files that will always appear as dead code.
# Option 1 (preferred): exclude by file — keeps dead_code detection active
# for the rest of the codebase.
exclude_files = [
    "src/kani_proofs.rs",
    "src/testutil.rs",
]

[duplicates]
detect_dead_code = true   # still active for non-excluded files

[complexity]
max_cognitive = 18
max_cyclomatic = 11
max_function_lines = 68

[srp]
max_parameters = 5
file_length_baseline = 300
file_length_ceiling = 800
lcom4_threshold = 2

[weights]
iosp         = 0.22
complexity   = 0.18
dry          = 0.13
srp          = 0.18
coupling     = 0.09
test_quality = 0.10
architecture = 0.10
```

---

## Shape 4: Workspace with integration test binary

```
my-project/
├── Cargo.toml
├── rustqual.toml
├── crates/
│   └── core/
├── tests/
│   └── integration/        # integration test binary — not a crate
│       └── main.rs
```

Rustqual workspace scan includes `tests/` files. Integration test files will have
high TQ_UNTESTED counts (they call the SUT but aren't themselves tested).

```toml
# rustqual.toml
ignore_functions = ["main"]

# Exclude integration test directories from workspace scan.
exclude_files = [
    "tests/integration/main.rs",
    "tests/",
]

[duplicates]
detect_dead_code = false

[complexity]
max_cognitive = 18
max_cyclomatic = 11
max_function_lines = 68

[srp]
max_parameters = 5
file_length_baseline = 300
file_length_ceiling = 800
lcom4_threshold = 2

[weights]
iosp         = 0.22
complexity   = 0.18
dry          = 0.13
srp          = 0.18
coupling     = 0.09
test_quality = 0.10
architecture = 0.10
```

---

## Inline suppression reference (workspace context)

```rust
// I/O boundary — file existence check + read is inherently mixed
// qual:allow(iosp) reason: "I/O boundary: checks existence before reading"
pub fn open(path: &Path) -> Result<Config> {
    if !path.exists() { return Err(...); }
    let raw = fs::read_to_string(path)?;
    parse_config(&raw)
}

// CLI integration root — dispatch wiring, not domain logic
// qual:allow(iosp) reason: "integration root: CLI dispatch"
fn run(args: Args) -> Result<()> {
    match args.command {
        Command::Init => init::run(&args.config),
        Command::Sync => sync::run(&args.config),
    }
}
```

**Note**: `qual:allow(dry)` does NOT suppress `DEAD_CODE`. Use `detect_dead_code = false`
in config for cross-crate false positives — inline suppression is not available for
this finding type.
