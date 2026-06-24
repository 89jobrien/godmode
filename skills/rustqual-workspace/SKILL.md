---
name: "rustqual-workspace"
description: >
  rustqual guidance for Rust workspaces with multiple crates (lib + binary, lib + CLI,
  multi-crate workspace). Use when rustqual findings don't make sense in a workspace
  context — DEAD_CODE on pub API, TQ_UNTESTED on CLI dispatch, score mismatch between
  workspace-level and per-crate scans, or ORPHAN_SUPPRESSION after adding qual:allow(dry).
  Triggers on: "rustqual workspace", "dead code false positive", "cross-crate",
  "lib and binary crate", "qual:allow dry not working", "ORPHAN_SUPPRESSION",
  or any rustqual scenario involving pub API used across crate boundaries.
color: orange
model: inherit
---

# rustqual-workspace

Workspace-specific guidance for rustqual. rustqual operates per-crate — it cannot
trace cross-crate call graphs. This creates predictable false positives in workspaces
that require configuration rather than code changes.

For full rustqual reference (finding codes, CLI flags, config schema, quality loop),
see the `rustqual` skill. This skill covers only the workspace-specific patterns.

## The Core Limitation

rustqual analyzes one crate at a time. It cannot see:

- Functions in `lib-crate` called only by `binary-crate`
- Functions called by integration tests in `tests/`
- Functions called by other workspace members

This produces **structural false positives** that are correct findings within a single
crate but wrong in the workspace context.

---

## Pattern 1: DEAD_CODE on pub API

### Symptom

```
DEAD_CODE  src/lib.rs:20  node_count  (testonly)
DEAD_CODE  src/lib.rs:25  edge_count  (testonly)
```

These functions are `pub`, used by another crate, but rustqual only sees the unit
tests in this crate calling them — hence `(testonly)`.

### Wrong fix

Adding `// qual:allow(dry)` above the functions does NOT suppress DEAD_CODE. It
suppresses duplicate/boilerplate DRY findings only. The result:

```
ORPHAN_SUPPRESSION  src/lib.rs:19  qual:allow(dry) has no matching finding
```

### Correct fix

Disable dead code detection in `rustqual.toml`:

```toml
[duplicates]
detect_dead_code = false
```

Only do this when **all** DEAD_CODE findings are cross-crate false positives. If
some are genuine dead code, use `exclude_files` to carve out false-positive files
instead:

```toml
exclude_files = ["src/graph_api.rs"]  # pub API consumed by graph-cli crate
```

---

## Pattern 2: TQ_UNTESTED on CLI dispatch

### Symptom

```
TQ_UNTESTED  src/main.rs:12  run
TQ_UNTESTED  src/cli.rs:45  dispatch_command
```

CLI dispatch functions are pure wiring — they call library functions but contain no
testable logic themselves. Integration tests in `tests/` cover them end-to-end, but
rustqual cannot see those tests.

### Correct fix

Add the dispatch functions to `ignore_functions` in `rustqual.toml`:

```toml
ignore_functions = ["main", "run", "dispatch_command", "dispatch_*"]
```

Do not move these into the library crate to make them testable — that inverts the
architecture. CLI dispatch belongs in the binary crate.

---

## Pattern 3: Score Mismatch (Workspace vs Per-Crate)

### Symptom

```bash
rustqual .              # reports 74.2%
rustqual crates/core    # reports 91.0%
rustqual crates/cli     # reports 68.0%
# 91 + 68 ≠ 74, and neither matches the workspace total
```

### Why this happens

Workspace scan (`rustqual .`) includes:

- All `.rs` files at the workspace root
- Re-export shims
- Files not inside any crate's `src/`

Per-crate scans only include that crate's `src/`. Per-crate totals do not sum to the
workspace total.

### Rule

Pick one scope and stick with it. Use `--save-baseline` to lock the method:

```bash
rustqual . --save-baseline .ctx/rustqual-baseline.json      # workspace scope
# OR
rustqual crates/core --save-baseline .ctx/core-baseline.json  # per-crate scope
```

Never compare a workspace score against a per-crate score — they measure different
things.

---

## Pattern 4: Lib + Binary Split — TQ_UNTESTED in Lib

### Symptom

```
TQ_UNTESTED  src/lib.rs:88  parse_config
TQ_UNTESTED  src/lib.rs:102  validate_schema
```

These functions are in the lib crate but have no unit tests there. Only the binary
crate exercises them through integration.

### Correct fix

Add unit tests in the lib crate. This is the real fix — these functions are testable
in isolation and should be tested there:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_config_valid_input_returns_ok() {
        let result = parse_config("key = \"value\"");
        assert!(result.is_ok());
    }
}
```

Do NOT add dispatch functions to `ignore_functions` for lib crate functions. Lib
functions can and should have unit tests.

---

## Workspace rustqual.toml Template

```toml
# Workspace-level config — place in repo root
ignore_functions = ["main", "run", "dispatch_*"]

[duplicates]
# Set false only if ALL dead-code findings are cross-crate pub API false positives.
# Leave true if any genuine dead code exists.
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

For per-crate configs, place a `rustqual.toml` in each crate directory. Per-crate
configs override the workspace config for that crate's scan.

---

## Decision Tree: Workspace Finding Triage

```
Finding: DEAD_CODE
  └─ Is the function pub?
       ├─ Yes → Is it used by another crate?
       │          ├─ Yes → detect_dead_code = false (or exclude_files)
       │          └─ No  → Real dead code. Remove or add a test.
       └─ No  → Real dead code. Remove.

Finding: TQ_UNTESTED
  └─ Which crate?
       ├─ Binary/CLI → Is it dispatch wiring?
       │                 ├─ Yes → ignore_functions
       │                 └─ No  → Move logic to lib crate + add unit test
       └─ Lib        → Add a unit test. No exception.

Finding: ORPHAN_SUPPRESSION after qual:allow(dry)
  └─ You tried to suppress DEAD_CODE with qual:allow(dry).
     Remove the qual:allow annotation.
     Use detect_dead_code = false instead.
```
