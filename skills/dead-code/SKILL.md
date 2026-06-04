---
name: "godmode:dead-code"
description: >
  Dead code detector for Rust workspaces. Finds unused public API surface, orphaned
  test helpers, stale feature flags, and plugin artifacts referencing removed CLI
  subcommands. Cross-references pub exports against call sites across the workspace.
requires: []
next: []
---

# Dead Code Detection

Find unused public API surface, test helpers, feature flags, and stale references
without relying solely on compiler warnings.

## When to Use

- Asked to "find dead code", "unused exports", "clean up API surface", "find unused"
- Preparing a breaking change or major refactor
- Before removing a crate or module — check for orphaned references
- Auditing plugin integrity (skills/agents referencing removed subcommands)
- Periodic hygiene sweep to identify candidates for removal or visibility change

## Detection Categories

### Public API Surface

Enumerates all `pub fn`, `pub struct`, `pub enum`, `pub trait`, `pub const`, and `pub
static` in a crate or workspace. For each, cross-references via Grep to find:

- Direct callers outside the defining crate
- Type constructor sites
- Trait implementations
- Module re-exports

Reports:

- Zero external callers → candidate for `pub(crate)` or removal
- Only test callers → test utility, consider test-module-only visibility
- Only macro-expanded callers → higher risk of false negatives

### Orphaned Test Helpers

Functions and macros defined in `#[cfg(test)]` modules that:

- Are not called by any test
- Are not re-exported for integration tests
- Appear to be scaffolding from abandoned test patterns

Reports as Nitpick severity — may be kept for quick development debugging.

### Feature Flags

Reads `[features]` in `Cargo.toml`. For each, searches the codebase for:

```rust
#[cfg(feature = "flag_name")]
```

Reports:

- Feature declared but no `cfg(feature)` usage → dead feature gate
- Feature guard protecting code that's never called anyway → nested dead code

### Plugin Artifacts

For godmode plugin skills and agents:

- Scan `agents/*.md` and `skills/**/*.md` for `godmode <subcommand>` references
- Cross-reference against the CLI help/CLAUDE.md reference
- Flag references to nonexistent subcommands as Blocking

Example: skill references `godmode task pull --github` but that flag was removed in
v0.4.0.

## Output Format

### Blocking

**Definite problems** — code/references broken by removal or mutation:

- Public function only called by code in a deleted module
- Trait implementation for a type that no longer exists
- Agent/skill referencing a nonexistent godmode subcommand
- Feature gate protecting imported-but-removed dependency

### Suggestion

**Candidates for cleanup** — unused but not broken:

- Pub function with zero external callers (keep if part of intended API contract)
- Feature flag with zero cfg usage (may be vestigial from earlier design)
- Pub type never instantiated outside tests (visibility could be reduced)

### Nitpick

**Minor hygiene** — does not block anything, low priority:

- Unused test helpers (kept for quick prototyping, low signal)
- Code behind always-false cfg gates (benign but can be cleaned up)
- Orphaned re-exports
- Deprecated functions without removal timeline

## Example Detection Workflow

```
1. List all pub exports in crates/foo/src/lib.rs
   → [Line 42] pub fn process_item(...)
   → [Line 18] pub struct Config { ... }

2. Grep for "process_item" across workspace
   → 0 results outside crates/foo
   → 2 results in crates/foo/tests/
   → Verdict: test-only, suggest pub(crate) or move to test module

3. Grep for "Config" across workspace
   → 5 callers in crates/bar
   → 3 callers in crates/baz
   → Verdict: actively used external API

4. List features in crates/foo/Cargo.toml
   → [features] default = [], experimental, vendored

5. Grep for cfg(feature = "experimental")
   → 0 results
   → Verdict: dead feature flag, candidate for removal

6. Check agents/ for subcommand refs
   → agents/example.md: "godmode task pull-github"
   → CLAUDE.md: no "task pull-github" subcommand
   → Verdict: Blocking — agent references nonexistent subcommand
```

## Limitations and Caveats

**False Negatives** — Grep may miss:

- Calls via dynamic dispatch (HashMap of function pointers, vtables)
- Calls via reflection or macro-generated code
- Calls from FFI or other language bindings
- Calls where the function is aliased (`let f = process_item; f()`)

If a public export appears unused but you suspect dynamic dispatch, manually review
the call site.

**False Positives** — Grep may report:

- String literal matches (e.g., function name in a log message or doc example)
- Comments referencing the function
- Matches in generated or vendored code (bindings, test fixtures)

Use `-w` (word boundary) and `--type rs` (language filtering) to reduce noise.

## No Fix Mode

This skill is **read-only**. Never:

- Delete functions or types
- Modify visibility (change `pub` to `pub(crate)`)
- Remove feature gates or feature flags from Cargo.toml
- Edit agent/skill files to fix subcommand references
- Run `cargo remove` or edit `Cargo.toml` programmatically

Report findings with context (last commit, usage pattern, confidence level) and let the
user decide what to remove.

## See Also

- `gm-code-review-agent` — detects dead code as part of multi-dimensional review
- `godmode review agents` — checks plugin artifact integrity
- Rust compiler warning: `#[warn(dead_code)]` — catches some unused functions (this
  skill finds unused _public_ surface the compiler allows)
