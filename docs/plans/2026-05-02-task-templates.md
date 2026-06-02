# Plan: Task Templates (`godmode task apply`)

**Status**: done

## Goal

Allow users to define reusable task bundles as YAML files with `{{var}}` substitution.
`godmode task apply <name> --var key=value` expands a template and ingests the resulting
tasks into the active graph. Eliminates repetitive manual task setup across sessions.

## Architecture

**Crates affected:** `godmode-core` (new module), `godmode-cli` (two new subcommand actions)

**New files:**

| File                                   | Role                                            |
| -------------------------------------- | ----------------------------------------------- |
| `crates/godmode-core/src/templates.rs` | Template resolution, substitution, apply logic  |
| `templates/`                           | User-defined template directory (repo-local)    |
| `~/.config/godmode/templates/`         | Global template directory (cross-repo fallback) |

**Modified files:**

| File                             | Change                                           |
| -------------------------------- | ------------------------------------------------ |
| `crates/godmode-core/src/lib.rs` | `pub mod templates;`                             |
| `crates/godmode-cli/src/main.rs` | `TaskAction::Apply`, `TaskAction::ListTemplates` |

### Template file format

`templates/<name>.yaml`:

```yaml
meta:
  name: tdd-cycle
  description: "Red-green-refactor loop for one crate"
  vars:
    - name: crate
      required: true
    - name: prefix
      default: "t"

tasks:
  - id: "{{prefix}}-red"
    title: "Write failing test for {{crate}}"
    crate_name: "{{crate}}"
    run: "cargo nextest run -p {{crate}}"
  - id: "{{prefix}}-green"
    title: "Implement minimum code to pass"
    crate_name: "{{crate}}"
    depends_on: ["{{prefix}}-red"]
  - id: "{{prefix}}-refactor"
    title: "Refactor with tests green"
    crate_name: "{{crate}}"
    run: "cargo nextest run -p {{crate}}"
    depends_on: ["{{prefix}}-green"]
```

`{{var}}` substitution is applied as `str::replace` over the raw YAML string before
parsing. Required vars without a supplied value produce an error before any graph mutation.

### Core module public API (`templates.rs`)

```rust
pub struct TemplateMeta {
    pub name: String,
    pub description: String,
}

pub struct Template {
    pub meta: TemplateMeta,
    pub tasks: Vec<Task>,
}

/// Locate a template by name. Checks local `templates/` first, then global.
pub fn find(root: &Path, name: &str) -> Result<PathBuf>

/// Load a template file, substitute vars, and return resolved Template.
/// `vars` is a slice of "key=value" strings.
pub fn load(path: &Path, vars: &[String]) -> Result<Template>

/// Apply a resolved template into a graph. Idempotent — skips existing task IDs.
/// Returns (applied, skipped) counts.
pub fn apply(graph: &mut TaskGraph, template: Template) -> Result<(usize, usize)>
```

No new external dependencies. Global dir resolved via `std::env::var("HOME")`.

### CLI subcommands

```
godmode task apply <name> [--var key=value]...
godmode task list-templates
```

`task apply` output:

- Human: `Applied 3 task(s) from template 'tdd-cycle'. (0 skipped)`
- JSON: `{"ok":true,"applied":3,"skipped":0}`

`task list-templates` output:

- Human: table of name, description, source (local/global)
- JSON: `[{"name":"...","description":"...","source":"local"}]`

## Tech Decisions

- **`str::replace` substitution** — no templating engine. `{{var}}` is replaced in the
  raw YAML string before `serde_yaml::from_str`. Simple, zero deps, sufficient for v1.
- **Resolution order: local first, global fallback** — local `templates/` takes precedence
  over `~/.config/godmode/templates/`. Same name in both → local wins, no error.
- **Idempotent apply** — reuses `graph::add` which skips existing IDs silently. Re-running
  `task apply` on an already-populated graph is safe.
- **Required var validation before mutation** — all vars resolved and validated before
  any `graph::add` call. Partial graph mutation on error is not possible.
- **No `dirs` crate** — global dir resolved manually via `$HOME` to avoid a new dependency.

## Out of Scope

- Template inheritance or composition (`extends:` another template)
- Nested `{{var}}` expressions or conditional blocks
- `godmode template new` / `godmode template edit` subcommands
- Validation that `run:` commands exist on PATH
- Built-in template library shipped with the binary
- Template versioning or semver constraints
- Cross-template `depends_on` (tasks in one template depending on tasks from another)

---

## Tasks

### Task 1: `templates.rs` — core module

**File**: `crates/godmode-core/src/templates.rs`
**Run**: `cargo nextest run -p godmode-core`

Implement `find`, `load`, `apply`. Internal helper `substitute(raw: &str, vars: &HashMap<String,String>) -> Result<String>` iterates the vars map and calls `raw.replace`. Error if any `required: true` var has no supplied value. Parse substituted string with `serde_yaml::from_str::<RawTemplate>` where `RawTemplate` mirrors the file format (meta + tasks).

Verify: unit tests for substitution (happy path, missing required var, unknown var is a no-op), `find` resolution order, `apply` idempotency.

### Task 2: CLI — `task apply` and `task list-templates`

**Files**: `crates/godmode-cli/src/main.rs`
**Run**: `cargo nextest run -p godmode-cli`

Add `TaskAction::Apply { name: String, vars: Vec<String> }` and
`TaskAction::ListTemplates` to the `TaskAction` enum. Wire into the `match` block.
`Apply` calls `templates::find`, `templates::load`, `templates::apply`, `graph::save`.
`ListTemplates` calls a new `templates::list(root)` that returns
`Vec<(TemplateMeta, TemplateSource)>`.

Verify: integration test with a fixture template file confirming tasks appear in graph
after apply.

### Task 3: Conformance + verification

**Run**: `just conformance && cargo nextest run --workspace`

1. `cargo nextest run --workspace` — all green.
2. `cargo clippy --workspace -- -D warnings` — zero warnings.
3. `just conformance` — all checks pass.
4. Commit: `feat(godmode-core): task templates with var substitution`
