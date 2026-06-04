---
name: "godmode:workspace-refactor"
description: >
  Discover and catalog breaking changes when a shared crate's public API
  changes. Find all dependent repos, list call sites that break, estimate
  migration effort, and optionally dispatch parallel subagents to apply
  migrations.
requires: []
next: []
---

# Workspace Refactor

Coordinate cross-repo refactors when a shared crate (e.g., `devkit`, `crux`,
`minibox`) changes its public API.

## When to Use

- A shared crate in `~/dev/` has breaking changes to its public API
- You need to find and migrate all downstream dependents
- You want to parallelize the migration across multiple repos
- You need to estimate effort before committing to the work

## Discovery Phase

### 1. Identify the changed crate

Ask the user which crate changed and what changed:

- Renamed exports (traits, structs, enums, functions, modules)
- Removed exports (breaking deletion)
- Changed signatures (function arguments, return types, trait bounds)

Example:

```
Crate: devkit
Changes:
  - Renamed: AsyncExecutor → RuntimeExecutor
  - Removed: old_api::deprecated_fn()
  - Changed: Config::new() now requires &str arg (was Option<String>)
```

### 2. Find all dependent repos

For each repo under `~/dev/`:

```bash
# Check if the repo has a Cargo.toml
# Read Cargo.toml and look for the changed crate in [dependencies]
# or [dev-dependencies]
```

Tools:

- **Glob**: `~/dev/*/Cargo.toml` to enumerate all Rust projects
- **Read**: each Cargo.toml to check for the dependency
- **Bash**: optional, to parse TOML if needed

Record:

```
Dependents of devkit:
  - crux (direct dependency)
  - maestro (dev dependency)
  - braid (direct dependency)
  - (others...)
```

### 3. For each dependent, find call sites

For each dependent repo, use Grep to search for the changed symbols:

```bash
# In repo root:
# rg "AsyncExecutor" --type rs --type toml
# rg "old_api" --type rs
# rg "Config::new" --type rs
```

- Search for imports: `use <crate>::<symbol>`
- Search for type references: `<Symbol>`, `impl <Trait>`
- Search for function calls: `<function>()`, `<type>::method()`
- Search for config/annotation usage: `#[<attr>]`, `<Const>::VALUE`

Record each match:

```
crux/src/executor.rs:42
  Current: pub struct Executor(AsyncExecutor)
  Change: Rename AsyncExecutor to RuntimeExecutor

crux/src/config.rs:10
  Current: let cfg = Config::new(None)?
  Change: Config::new() now requires &str, not Option<String>
```

## Analysis Phase

### 4. Build per-repo migration checklist

For each dependent repo, produce a markdown checklist:

```markdown
## crux Migration (devkit API changes)

### Changes Required

| File                 | Line | Current                            | New                                  | Status |
| -------------------- | ---- | ---------------------------------- | ------------------------------------ | ------ |
| src/executor.rs      | 42   | use AsyncExecutor                  | use RuntimeExecutor                  | [ ]    |
| src/executor.rs      | 99   | pub struct Executor(AsyncExecutor) | pub struct Executor(RuntimeExecutor) | [ ]    |
| src/config.rs        | 10   | Config::new(None)                  | Config::new("default")               | [ ]    |
| tests/integration.rs | 55   | old_api::deprecated_fn()           | (delete or replace)                  | [ ]    |

### Summary

- Files affected: 3
- Call sites: 4
- Estimated effort: S (Small — mostly renames)
```

### 5. Estimate effort per repo

Count the breaking call sites and assess complexity:

- **S (Small)**: 1–3 simple renames or deletions
- **M (Medium)**: 4–10 call sites, some signature adjustments
- **L (Large)**: 11+ call sites, complex refactoring needed

Add effort estimate to each repo's checklist.

## Execution Phase (Optional)

If the user asks you to apply the migrations:

### 6. Dispatch subagents in parallel

Use the `parallel-agents` pattern (see helper or skill reference). For each
dependent repo:

1. Create a subagent worktree (e.g., `godmode worktree add <branch>`)
2. Pass the migration checklist to the subagent
3. Subagent applies changes:
   - Edit files according to checklist
   - Run `cargo check` to verify
   - Commit with descriptive message (e.g., "refactor: migrate to RuntimeExecutor")
   - Merge back to main and remove worktree

Cap concurrent subagents at 5 (per workspace guardrails).

### 7. Collect results

After all subagents finish:

```markdown
## Migration Complete

| Repo    | Status   | Files Changed | Commits          |
| ------- | -------- | ------------- | ---------------- |
| crux    | ✓        | 3             | abc1234          |
| braid   | ✓        | 2             | def5678          |
| maestro | ⚠ Manual | N/A           | None (conflicts) |

Total: 2/3 automated, 1 requires manual review
```

## Guardrails

- **Always dry-run first.** Show the migration plan, get approval, estimate
  effort before dispatching subagents.
- **Use `git -C <path>`**, never `cd <path> && git`. The tooling resets cwd on
  `cd`.
- **Respect per-repo CLAUDE.md.** Some repos have custom build steps, pre-commit
  hooks, or conventions. Check before modifying.
- **For multi-repo changes, propose systemic solution first.** Do not edit each
  repo individually if there is a shared abstraction or configuration option
  that can minimize churn.
- **Never force-push or hard-reset.** Use normal merge workflows.
- **Each subagent must check `git branch --show-current` before every commit.**
  Never commit to `main` directly.
- **Verify subagent completion.** A subagent that leaves a worktree unmerged or
  a commit uncommitted has not finished.
- **If a dependent is not a Rust project,** ask the user how to detect its
  dependencies (e.g., package.json for Node, go.mod for Go).

## See Also

- `parallel-agents` skill — dispatch multiple subagents with safeguards
- `code-review` skill — review the applied migrations before merge
