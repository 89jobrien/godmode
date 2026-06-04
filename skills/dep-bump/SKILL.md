---
name: "godmode:dep-bump"
description: >
  Propagate a workspace crate version bump across ~/dev/ downstream
  dependencies. Discover all pinned locations, update safely with Edit,
  verify with cargo check, and report pass/fail per repo.
requires: []
next: [cap]
---

# Dependency Bump Propagation

Propagate a bumped workspace crate version to all repos that depend on it
across ~/dev/. This is a companion to workspace-release-impact and
workspace-bump-commit skills.

## When to Use

- After a workspace crate (e.g., devkit, romp) reaches a new version
- When asked to "bump propagate", "propagate version", "update dependents"
- Before committing a version bump to ensure all downstream repos are aware
- When onboarding a new repo into the workspace dependency tree
- To align all downstream pins to a single version

## The Propagation Process

### Discovery Phase

**Goal**: Map all locations where the crate is pinned.

1. Identify the crate name, old version, and new version:
   - Crate name (e.g., "devkit")
   - Old version (e.g., "0.4.0")
   - New version (e.g., "0.5.0")

2. Search ~/dev/ for all Cargo.toml files:

   ```bash
   rg "<crate-name> = \"<old-version>\"" ~/dev --glob "Cargo.toml" -B 2 -A 2
   ```

3. Record each match:
   - Full repo path
   - File path (relative to repo root or absolute)
   - Line number
   - Current version pinned
   - Dependency type (normal, dev, build)

4. Group by repository:
   - Some repos may have multiple Cargo.toml files (workspace sub-crates)
   - Some repos may pin the crate in multiple places (unlikely but possible)

**Output**: List of repos to update, grouped by directory structure.

### Update Phase

**Goal**: Apply the version bump to all discovered locations.

**Precondition**: User confirms the update plan.

For each affected Cargo.toml:

1. Read the file fully using the Read tool
2. Locate the dependency line: `<crate-name> = "<old-version>"`
3. Use the Edit tool to replace with `<crate-name> = "<new-version>"`
4. Verify no other lines changed (Edit preserves surrounding context)

**Constraints**:

- Never update the crate's own version in its Cargo.toml
- Only update dependency version pins, not feature specifications
- If the dependency is yanked or not found, pause and report

**Example**:

```toml
# Old
devkit = "0.4.0"
# New
devkit = "0.5.0"
```

### Verification Phase

**Goal**: Ensure each repo still builds after the version bump.

For each updated repo:

1. Run cargo check on the manifest:

   ```bash
   cargo check --manifest-path <path-to-cargo-toml>
   ```

   Or for workspace roots:

   ```bash
   cargo check --workspace --manifest-path <path-to-cargo-toml>
   ```

2. Capture stdout, stderr, and exit code

3. Classify result:
   - **Pass**: exit code 0, no errors
   - **Fail**: exit code nonzero, compilation error
   - **Blocked**: unable to run (binary missing, permission denied)

4. For failures, collect the full error output:
   - Dependency resolution errors
   - Compilation errors in dependent code
   - Feature flag conflicts

**Important**: Do NOT run `cargo build` — checking is sufficient and faster.

### Reporting Phase

**Goal**: Summarize results per repo and identify manual interventions needed.

Create a table:

```
## Propagation Results: <crate> <old-version> → <new-version>

| Repository | Status | Notes |
|:---|:---:|:---|
| ~/dev/repo-1 | Pass | - |
| ~/dev/repo-2 | Fail | error: dependency version not found on crates.io |
| ~/dev/repo-3 | Pass | - |

**Summary**: 2 pass, 1 fail

### Passed Repos

No action needed. Repos are ready for commit and test.

### Failed Repos (Manual Intervention)

**repo-2**:
```

error: version `0.5.0` of `devkit` not found in registry

```

Next steps:
- Verify the new version exists on crates.io
- Check if the crate has been published (not just tagged locally)
- If not published, consider a dry run with `cargo tree` instead
```

## Integration with Other Skills

### workspace-release-impact (atelier)

Use after running workspace-release-impact to identify which repos are
affected. The impact skill shows the dependency graph; dep-bump propagates
the actual version update.

### workspace-bump-commit (atelier)

Run after successful propagation. The bump-commit skill creates a coordinated
commit message and changelog entry across all updated repos. Dep-bump
discovers and validates; bump-commit commits the changes.

## Guardrails

- **Always show the full update plan with line numbers and ask for explicit
  confirmation before editing any Cargo.toml.**
- Never speculatively update all Cargo.toml files matching a pattern — only
  update files where the crate version is explicitly pinned.
- Never update a crate's own version in its Cargo.toml. If the bump targets
  devkit and you find devkit/Cargo.toml with devkit as a member, skip it.
- Never bump major versions without understanding the breaking changes. The
  user must request major version bumps explicitly.
- Always verify the new version exists on crates.io before updating. Use
  `cargo search <crate>` or the registry web UI to confirm.
- For any repo where cargo check fails, show the exact error and pause —
  do not continue to the next repo until the user acknowledges the failure.
- If the version is yanked, report and ask the user to select a different
  version.
- Use Edit tool only — never use sed, stream replacements, or other text
  munging. Edit preserves formatting and context.
- For workspace-scoped crates, be sure to check both the workspace root and
  all sub-crate Cargo.toml files.
- Never assume feature flags are preserved during a version bump. If the
  update includes features (e.g., `devkit = { version = "0.5.0", features =
["foo"] }`), verify the feature exists in the new version.
