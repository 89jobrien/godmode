# Plan: `godmode review` + `godmode release` subcommand groups

## Overview

Add two new top-level subcommand groups to `godmode`:

- `godmode review` — plugin conformance and consistency auditing
- `godmode release` — version bump, tag, and push workflow

Both follow the existing noun-group pattern (`task`, `wave`, `worktree`).
All new logic lives in `godmode-core`; `godmode-cli` gets thin clap wiring only.

---

## Task 1: `review` module in `godmode-core`

**File**: `crates/godmode-core/src/review.rs`

### What it does

Wraps the conformance checks currently in `tests/conformance/plugin-structure.nu`
as Rust functions. Does **not** replace the nu script — `just conformance` remains
the canonical gate. This module makes the same checks accessible as `godmode review`.

### Structs

```rust
pub struct Finding {
    pub skill: String,   // skill dir name, "plugin.json", or "index"
    pub check: String,   // short label, e.g. "missing SKILL.md"
    pub message: String,
}

pub struct ReviewReport {
    pub checks: u32,
    pub findings: Vec<Finding>,
    pub passed: bool,
}
```

### Public API

```rust
/// Run all conformance checks (equivalent to `just conformance`).
pub fn run_all(root: &Path) -> Result<ReviewReport>

/// Check skill dirs: SKILL.md present, frontmatter name, index entries, link resolution.
pub fn check_skills(root: &Path) -> Result<ReviewReport>

/// Check agent frontmatter: name, model, tools fields all present and non-empty.
pub fn check_agents(root: &Path) -> Result<ReviewReport>
```

### Checks matrix

| Check                                                               | `review self` | `review skills` | `review agents` |
| ------------------------------------------------------------------- | :-----------: | :-------------: | :-------------: |
| Every skill dir has SKILL.md                                        |       x       |        x        |                 |
| Frontmatter `name:` non-empty                                       |       x       |        x        |                 |
| Skill name in `skill-index.md`                                      |       x       |        x        |                 |
| Skill name in `using-godmode/SKILL.md`                              |       x       |        x        |                 |
| No orphan index entries                                             |       x       |                 |                 |
| `references/` links resolve                                         |       x       |        x        |                 |
| `helpers/` links resolve                                            |       x       |        x        |                 |
| `plugin.json` allowed fields only                                   |       x       |                 |                 |
| CLI subcommand names valid                                          |       x       |        x        |                 |
| Cross-skill consistency (merge, concurrency, BLOCKED, branch guard) |       x       |        x        |                 |
| `_lib/*.nu` parse without error                                     |       x       |                 |                 |
| `_lib/` refs in SKILL.md resolve                                    |       x       |        x        |                 |
| Agent `name`, `model`, `tools` present                              |       x       |                 |        x        |

### Output format

Human (default):

```
78 checks passed.
```

or:

```
[cap] missing SKILL.md
[brainstorm:12] unknown subcommand: godmode foo
2 checks failed out of 78 total.
```

JSON (`--json`):

```json
{ "checks": 78, "passed": true, "findings": [] }
```

### Tests

Unit tests in `review.rs` using temp dirs:

- `all_checks_pass_on_clean_fixture` — minimal valid skill dir tree
- `detects_missing_skill_md`
- `detects_missing_frontmatter_name`
- `detects_orphan_index_entry`
- `detects_broken_references_link`
- `detects_agent_missing_model_field`

---

## Task 2: `release` module in `godmode-core`

**File**: `crates/godmode-core/src/release.rs`

### What it does

Manages the plugin release lifecycle: read current version from `plugin.json`,
increment it, write it back, create a git tag, and push to origin.

Reads `.version-bump.json` to discover which files to update (already lists
`.claude-plugin/plugin.json` with `"field": "version"`).

### Structs

```rust
pub struct ReleaseConfig {
    pub files: Vec<FileTarget>,   // from .version-bump.json
}

pub struct FileTarget {
    pub path: PathBuf,
    pub field: String,
}

pub struct ReleaseInfo {
    pub old_version: String,
    pub new_version: String,
    pub tag: String,
    pub pushed: bool,
}
```

### Public API

```rust
/// Read current version from plugin.json (or first file in .version-bump.json).
pub fn current_version(root: &Path) -> Result<String>

/// Increment patch component: "1.1.0" -> "1.1.1".
/// Accepts optional explicit version string to set directly.
pub fn bump(root: &Path, explicit: Option<&str>) -> Result<ReleaseInfo>

/// Create annotated git tag for the current version.
pub fn tag(root: &Path) -> Result<String>   // returns tag name

/// Push current branch + tag to origin.
pub fn push(root: &Path) -> Result<()>
```

### Version bump logic

1. Read `.version-bump.json` → collect `files` array
2. For each file: read JSON, update the named `field` with new version, write back
3. Default bump strategy: increment patch (`semver` crate or simple split on `.`)
4. Use `semver` if already in Cargo.toml deps; otherwise parse manually (no new dep)

### Tag format

`v{version}` — e.g. `v1.1.1`. Annotated tag with message `"release v1.1.1"`.

### Tests

- `bump_increments_patch` — temp dir with `plugin.json`, verifies version written
- `bump_accepts_explicit_version`
- `current_version_reads_plugin_json`
- `bump_fails_on_missing_version_bump_json`

---

## Task 3: CLI wiring in `godmode-cli`

**File**: `crates/godmode-cli/src/main.rs`

### New enum variants

```rust
/// Plugin conformance and consistency auditing.
Review {
    #[command(subcommand)]
    action: ReviewAction,
},

/// Plugin release: bump version, tag, push.
Release {
    #[command(subcommand)]
    action: ReleaseAction,
},
```

```rust
#[derive(Subcommand)]
enum ReviewAction {
    /// Run all conformance checks (skills + agents + plugin.json).
    Self_,
    /// Check skill dirs for SKILL.md, frontmatter, and link integrity.
    Skills,
    /// Check agent frontmatter completeness.
    Agents,
}

#[derive(Subcommand)]
enum ReleaseAction {
    /// Show current plugin version.
    Current,
    /// Increment patch version in all files listed in .version-bump.json.
    Bump {
        /// Set an explicit version instead of auto-incrementing.
        #[arg(long)]
        version: Option<String>,
    },
    /// Create annotated git tag for the current version.
    Tag,
    /// Push current branch and version tag to origin.
    Push,
}
```

### Match arms

Each arm calls the corresponding `godmode_core::review::*` or
`godmode_core::release::*` function and formats output per the `--json` flag,
following the same pattern as `Cmd::Verify`.

---

## Task 4: `lib.rs` exports

Add to `crates/godmode-core/src/lib.rs`:

```rust
pub mod release;
pub mod review;
```

---

## Task 5: conformance update

`tests/conformance/plugin-structure.nu` Check 8 (`canonical_subcommands` list)
must be extended with the new subcommands:

```
"review self"
"review skills"
"review agents"
"release current"
"release bump"
"release tag"
"release push"
```

---

## Execution order

```
t1  review module (godmode-core/src/review.rs) + unit tests
t2  release module (godmode-core/src/release.rs) + unit tests   [independent of t1]
t3  lib.rs exports                                               [depends: t1, t2]
t4  CLI wiring (main.rs)                                         [depends: t3]
t5  conformance update (plugin-structure.nu)                     [depends: t4]
```

Tasks t1 and t2 are independent and can be dispatched in parallel.

---

## Files touched

| File                                    | Change                            |
| --------------------------------------- | --------------------------------- |
| `crates/godmode-core/src/review.rs`     | new                               |
| `crates/godmode-core/src/release.rs`    | new                               |
| `crates/godmode-core/src/lib.rs`        | add 2 pub mod lines               |
| `crates/godmode-cli/src/main.rs`        | new enum variants + match arms    |
| `tests/conformance/plugin-structure.nu` | extend canonical_subcommands list |

No other files are touched.

---

## Acceptance criteria

- `just conformance` passes (all existing checks + new subcommand entries)
- `cargo nextest run --workspace` passes
- `cargo clippy --workspace -- -D warnings` clean
- `godmode review self` exits 0 on clean repo, exit 1 with findings listed on failure
- `godmode review skills` and `godmode review agents` work independently
- `godmode release current` prints current version from `plugin.json`
- `godmode release bump` updates `plugin.json` version and prints old→new
- `godmode release tag` creates `v{version}` annotated tag
- `godmode release push` pushes branch + tag
- All commands support `--json`
