# Plan: `godmode graph build` — Interactive Graph Construction

**Status**: done

## Goal

A `godmode graph build` subcommand that guides a human through constructing and evolving
a task graph in three phases: shape, wire, validate. Interactive by default (one question
at a time, tasks persisted incrementally). Non-interactive via `--input <file>` using the
existing template YAML format.

## Architecture

**Crates affected:** `godmode-core` (new module), `godmode-cli` (new top-level subcommand)

**New files:**

| File                                 | Role                                                |
| ------------------------------------ | --------------------------------------------------- |
| `crates/godmode-core/src/builder.rs` | Phase logic, input parsing, interactive prompt loop |

**Modified files:**

| File                             | Change                                 |
| -------------------------------- | -------------------------------------- |
| `crates/godmode-core/src/lib.rs` | `pub mod builder;`                     |
| `crates/godmode-cli/src/main.rs` | `Cmd::Graph` with `GraphAction::Build` |

## Behaviour

### Interactive mode (`godmode graph build`)

Runs the three-phase loop on stdin/stdout. Each phase asks one question at a time.
Each confirmed answer immediately calls `graph::add` and `graph::save` — the graph
is persisted after every task addition.

**Phase 1: Shape**

```
What's the next thing that needs to happen? (blank to finish)
> implement model layer
Which crate? (blank to skip)
> godmode-core
Task ID [t1]:
> [enter]
Added [t1] implement model layer (godmode-core)

What's the next thing that needs to happen? (blank to finish)
> ...
```

**Phase 2: Wire**

For each task added in Phase 1:

```
Does anything need to be done before [t2] "write tests"? (comma-separated IDs, blank to skip)
> t1
Updated [t2] depends_on: [t1]
```

After all deps wired, runs `godmode dispatch --critical-path` and prints result.

**Phase 3: Validate**

Runs and prints:

```bash
godmode status
godmode dispatch --critical-path
godmode task next
```

Checks for and surfaces:

- No runnable tasks (all blocked or empty)
- Over-wide graph (>5 independent roots)
- Single critical path (everything sequential — suggest parallelizing)
- Orphaned tasks (pending, deps done, not in `next`)

Each finding: one sentence + one suggested fix command. User confirms or skips.

Exit: prints `Graph ready. Run: godmode task next` and exits 0.

### Non-interactive mode (`godmode graph build --input <file>`)

Input file is a template YAML (same format as `templates/<name>.yaml`). `{{var}}`
substitution applied via `--var key=value` flags, same as `task apply`.

The file's `tasks` list is ingested directly — Phase 1 and 2 are driven by the file.
Phase 3 (validate) still runs and prints findings, but does not prompt — exits 0 if
graph is sound, exits 1 with findings printed to stderr if not.

```bash
godmode graph build --input templates/tdd-cycle.yaml --var crate=godmode-core
godmode graph build --input templates/tdd-cycle.yaml --var crate=foo --var prefix=foo
```

### `--json` flag

Emits a summary after all phases:

```json
{
  "added": 3,
  "wired": 2,
  "findings": [],
  "next": ["t1"]
}
```

## Tech Decisions

- **Incremental persistence** — `graph::save` called after every `graph::add` in
  interactive mode. No rollback on exit — partial graphs are valid and useful.
- **Stdin/stdout only** — no TUI library. Plain line-based prompts via `std::io`.
  Keeps the binary small and scriptable.
- **Non-interactive reuses template loader** — `builder::build_from_file` calls
  `templates::load` and `templates::apply` directly. No duplication.
- **Phase 3 is always run** — even in non-interactive mode. Validation findings are
  informational (not blocking) unless graph has zero runnable tasks (exits 1).
- **No new external dependencies** — stdin interaction via `std::io::BufRead`,
  no readline or TUI crate.

## Out of Scope

- Editing or removing tasks interactively (use `godmode task remove` / `godmode task block`)
- Undo / redo within the session
- Graph visualization as diagram
- Multi-graph sessions
- Auto-generating task titles from git log or issue titles

---

## Tasks

### Task 1: `builder.rs` — phase logic

**File**: `crates/godmode-core/src/builder.rs`
**Run**: `cargo nextest run -p godmode-core`

Implement:

```rust
/// Drive interactive graph construction on stdin/stdout.
pub fn build_interactive(root: &Path) -> Result<BuildSummary>

/// Drive non-interactive graph construction from a template file.
pub fn build_from_file(root: &Path, path: &Path, vars: &[String]) -> Result<BuildSummary>

pub struct BuildSummary {
    pub added: usize,
    pub wired: usize,
    pub findings: Vec<String>,
    pub next: Vec<String>,
}
```

`build_interactive` reads lines from `stdin`, prompts to stdout. Each confirmed task
calls `graph::add` + `graph::save` immediately.

`build_from_file` calls `templates::load` + `templates::apply`, then runs Phase 3
validation, returns `BuildSummary`.

Phase 3 validation logic extracted as `pub fn validate(graph: &TaskGraph) -> Vec<String>`
— returns finding strings, used by both interactive and non-interactive paths.

Verify: unit tests for `validate` (each finding condition), `build_from_file` with a
fixture YAML.

### Task 2: CLI — `godmode graph build`

**File**: `crates/godmode-cli/src/main.rs`
**Run**: `cargo nextest run -p godmode-cli`

Add top-level `Cmd::Graph { action: GraphAction }` and:

```rust
enum GraphAction {
    Build {
        #[arg(long)]
        input: Option<String>,
        #[arg(long, value_delimiter = ',')]
        var: Vec<String>,
    },
}
```

Wire `GraphAction::Build`:

- `input` present → `builder::build_from_file`
- `input` absent → `builder::build_interactive`
- Print `BuildSummary` human/JSON depending on `--json` flag
- Exit 1 if `summary.findings` contains a blocking finding (zero runnable tasks)

### Task 3: Conformance + verification

**Run**: `just conformance && cargo nextest run --workspace`

1. `cargo nextest run --workspace` — all green.
2. `cargo clippy --workspace -- -D warnings` — zero warnings.
3. `just conformance` — all checks pass.
4. Commit: `feat(godmode-core): graph build command — interactive + file-driven`
