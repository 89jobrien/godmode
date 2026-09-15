# Design: Feature Idea Tracking and Independent Slices

## Goal

Track all thirteen evidence-backed feature ideas with issue-linked source annotations, then deliver
Q1, Q3, and Q5 as independent, reviewable changes.

## Approved Approach

Merge one tracking change first, then branch Q1, Q3, and Q5 independently from the resulting main.

## Tracking Change

Each idea receives one GitHub issue and one `TODO(#N)` annotation at its primary implementation
boundary. The tracking change contains no behavior changes. Q1, Q3, and Q5 remove their annotations
when implemented; the other ten annotations remain until their issues are resolved.

| ID  | Annotation boundary                                                           |
| --- | ----------------------------------------------------------------------------- |
| Q1  | `crates/godmode-core/src/pipeline.rs` near `run_tasks` state initialization   |
| Q2  | `xtask/src/main.rs` next to the workspace nextest CI gate                     |
| Q3  | `crates/godmode-cli/src/main.rs` on `TaskAction::Add`                         |
| Q4  | `deny.toml` in the target-specific dependency graph configuration             |
| Q5  | `commands/gm/generate.nu` at the target-specific generation logic             |
| M1  | `crates/godmode-core/src/hooks/mod.rs` at the hook dispatch boundary          |
| M2  | `crates/godmode-core/src/workflow.rs` at state initialization and persistence |
| M3  | `crates/godmode-cli/src/main.rs` beside the top-level command enum            |
| M4  | `crates/godmode-core/src/integrations/doob.rs` beside outbound todo arguments |
| M5  | `.github/workflows/conformance.yml` beside the current event triggers         |
| L1  | `crates/godmode-core/src/pipeline.rs` beside step execution                   |
| L2  | `commands/gm/generate.nu` at workflow command generation                      |
| L3  | `crates/godmode-cli/src/main.rs` beside the top-level command enum            |

## Q1: Pipeline Run Entry Point

### Crate Ownership

- **Owner:** `godmode-core` owns pipeline state selection and execution.
- **Adapter:** `godmode-cli` passes the parsed `--from` value into the core operation.

### Public API

```rust
pub fn run_tasks(
    root: &Path,
    pipeline_name: &str,
    from: Option<&str>,
    fail_fast: bool,
) -> Result<RunResult>;
```

This replaces the existing three-argument function. No compatibility wrapper is added because the
workspace is pre-1.0 and all known callers are in this repository.

### Data Flow

1. Clap parses `pipeline run <NAME> --from <SKILL>` into `PipelineAction::Run`.
2. The CLI pipeline handler passes `from.as_deref()` to `pipeline::run_tasks`.
3. `run_tasks` resumes matching active state unchanged, or passes `from` to `start` for fresh state.
4. Existing state persistence and task execution continue without format changes.

### Verification

- Fresh run without `--from` starts at the first step.
- Fresh run with `--from` starts at the named entry point.
- Invalid entry points return the existing validation error.
- Matching active state resumes at its saved step even when `--from` is supplied.

## Q3: Task Add Metadata

### Crate Ownership

- **Owner:** `godmode-cli` owns argument parsing and mapping into the existing core `Task` model.
- **Affected core API:** none; `Task` already contains every required field.

### CLI Interface

```text
godmode task add <TITLE> \
  [--id <ID>] \
  [--depends-on <IDS>] \
  [--crate-name <CRATE>] \
  [--notes <TEXT>] \
  [--run <COMMAND>] \
  [--priority <high|normal|low>] \
  [--tag <TAG>]...
```

`--tag` is repeatable and preserves argument order. Existing invocations produce the same default
task: empty notes, no run command, normal priority, and no tags.

### Data Flow

1. Clap parses optional metadata into `TaskAction::Add`.
2. The task handler creates `Task::new` and assigns the supplied existing fields.
3. `Session::add_task` validates and persists the unchanged task schema.

### Verification

- CLI help exposes all four metadata options and priority values.
- A task created with every option serializes the supplied metadata.
- Repeated tags preserve order.
- Existing minimal task creation remains unchanged.

## Q5: Dual Repo-Local Command Projection

### Ownership

- **Owner:** `commands/gm/generate.nu` owns repository-local generation from command YAML.
- **Existing source:** `commands/gm/*.yaml` remains canonical for both target projections.

### Destinations

| Target      | Repository-local output |
| ----------- | ----------------------- |
| Claude Code | `commands/`             |
| OpenCode    | `.opencode/commands/`   |

The wrapper invokes the existing CLI once per target. It stops immediately if either generation
fails. Each invocation retains the projection layer's transactional write behavior.

### Data Flow

1. `generate.nu` reads canonical YAML from `commands/gm/`.
2. It renders Claude commands into `commands/` with the existing frontmatter.
3. It renders OpenCode commands into `.opencode/commands/` with OpenCode frontmatter and command
   references.
4. Conformance checks regenerate both targets in temporary directories and compare tracked output.

### Verification

- The wrapper produces equal command-name sets for both targets.
- Claude files retain `allowed-tools`; OpenCode files retain `subtask: false`.
- Slash-command references use `/gm:` for Claude and `/gm-` for OpenCode.
- Drift in either tracked projection fails conformance.

## Hexagonal Boundaries

- Q1 and Q3 add no external I/O boundary; they compose existing domain and session APIs.
- Q5 keeps target transforms in the existing Nushell generator. No external service or new port is
  required.

## Out of Scope

- Implementing Q2, Q4, M1-M5, or L1-L3 beyond issue creation and TODO annotation.
- Executing `parallel_with` or changing pipeline optional-step semantics.
- Installing commands into user-global Claude or OpenCode configuration.
- Changing the task YAML schema or JSON success response.
- Adding dependencies or feature flags.

## Risk

- [x] Breaking API change: Q1 changes `run_tasks`; all repository callers and API docs update in the
      same slice.
- [ ] Serialization change: none.
- [ ] New external dependency: none.
- [ ] New feature flag: none.
- [x] Cross-client projection drift: covered by target-specific conformance comparisons.
