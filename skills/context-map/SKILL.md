---
name: "godmode:context-map"
description: >
  Generate a map of all files relevant to a task before making any changes. Use this skill
  at the start of any implementation task — before writing a plan, before opening a file to
  edit, before dispatching subagents. Identifies files to modify, affected crates, dependency
  edges, test coverage, and reference patterns. Output feeds directly into the Architecture
  section of a godmode plan.
requires: []
next: [writing-plans]
---

# Context Map

Before implementing any changes, map the codebase. A context map answers: what exists, what
needs to change, what depends on it, and what tests cover it. Writing a plan without this
produces incomplete task lists and missed dependencies.

## When to Run

- At the start of any multi-file or multi-crate task
- Before writing a `godmode:writing-plans` plan
- Before dispatching parallel agents — slot assignments depend on crate boundaries
- When picking up a task in an unfamiliar part of the codebase

---

## Step 1: Locate Directly Relevant Files

Search for the types, functions, modules, and traits named in the task:

```nu
# Search for a type or function name across all Rust files
rg "TypeName|fn_name|mod_name" --type rust -l

# Search for a trait implementation
rg "impl TraitName" --type rust -l

# Find where a module is declared vs used
rg "^mod context|use .*context" --type rust -l

# Search across all file types if needed (configs, YAML, SQL)
rg "keyword" -l
```

List every file that declares, implements, or directly references the thing you are changing.
These are candidates for **Files to Modify**.

---

## Step 2: Trace Dependency Edges

For each file identified in Step 1, find what imports from it and what it imports:

```nu
# Who imports this module?
rg "use godmode_core::context" --type rust -l

# What does this file depend on? (scan its use declarations)
rg "^use " crates/godmode-core/src/context.rs

# Find re-exports that might hide consumers
rg "pub use.*context" --type rust -l
```

These are **Dependencies** — files that may need updates when the modified file changes its
public API. Mark any that re-export or wrap the changed type.

---

## Step 3: Find Test Coverage

```nu
# Find tests that exercise the relevant module
rg "use.*context|context::" --type rust crates/ | rg "#\[test\]|#\[tokio::test\]" -l

# Broader: find test files/modules touching the keyword
rg "context" --type rust crates/ -l | rg "tests?|spec"

# In a workspace, check godmode-conformance too
rg "context" tests/ -l 2>/dev/null
```

For each test file found, note what behaviour it exercises. Flag any gap: functionality
you intend to change that has no corresponding test.

---

## Step 4: Find Reference Patterns

Look for similar implementations already in the codebase to follow:

```nu
# Find other modules with the same shape (e.g. all modules that impl a Serialize struct)
rg "#\[derive.*Serialize" --type rust -l | head -10

# Find existing CLI subcommands as reference for a new one
rg "pub fn run" crates/godmode-cli/src/ -l

# Find similar integration patterns
rg "std::process::Command" --type rust -l
```

These are **Reference Patterns** — existing code that shows the project's conventions. Prefer
matching the nearest similar module over inventing new patterns.

---

## Step 5: Assess Risk

Answer each of these before declaring the map complete:

- **Public API change?** — does the modification change a `pub` function/struct/trait signature?
  If yes, all consumers (Step 2) must be updated in the same plan.
- **Serialization format change?** — any `#[derive(Serialize, Deserialize)]` struct that
  changes shape may invalidate persisted state files (`.ctx/godmode/tasks.yaml`, trace JSONL).
- **CLI output change?** — any subcommand output change may break callers using `--json`.
  Check `godmode context --json` consumers in hook scripts and agents.
- **Cross-crate boundary?** — changes that cross from `godmode-core` to `godmode-cli` or
  into `godmode-conformance` need tasks in multiple crates with explicit ordering.

---

## Output Format

Write the context map as a `## Context Map` section at the top of the plan, before Architecture:

```markdown
## Context Map

### Files to Modify

| File                                         | Purpose               | Changes Needed                  |
| -------------------------------------------- | --------------------- | ------------------------------- |
| `crates/godmode-core/src/context.rs`         | SessionContext struct | add `blocked_count` field       |
| `crates/godmode-cli/src/commands/context.rs` | CLI handler           | emit new field in --json output |

### Dependencies (may need updates)

| File                             | Relationship                                  |
| -------------------------------- | --------------------------------------------- |
| `crates/godmode-core/src/lib.rs` | re-exports `SessionContext`                   |
| `hooks/session-start.nu`         | calls `godmode context --json`, parses output |

### Test Coverage

| Test                                          | Covers                  |
| --------------------------------------------- | ----------------------- |
| `crates/godmode-core/src/context.rs` (inline) | `SessionContext::build` |
| `tests/conformance/src/context.rs`            | `--json` output shape   |

### Reference Patterns

| File                               | Pattern to Follow                          |
| ---------------------------------- | ------------------------------------------ |
| `crates/godmode-core/src/cache.rs` | struct serialised to JSON file, same shape |

### Risk

- [ ] `SessionContext` is `pub` — all consumers listed above must be updated
- [ ] `--json` output changes: update conformance test fixture
- [ ] No migration needed — field is additive with `#[serde(default)]`
```

---

## Checklist

Before handing the map to the plan author:

- [ ] Every file in **Files to Modify** was found by search, not assumed
- [ ] All `pub` API consumers are listed in **Dependencies**
- [ ] Test coverage gaps are flagged explicitly (not silently omitted)
- [ ] Risk items are concrete — no "may need changes" without naming the file
- [ ] Map was produced from the current HEAD, not from memory or stale context

---

## Related

- `skills/writing-plans/SKILL.md` — context map feeds into Plan Architecture section
- `skills/parallel-agents/SKILL.md` — crate boundaries from the map drive slot assignments
- `skills/systematic-debugging/SKILL.md` — same search techniques applied to bug tracing
