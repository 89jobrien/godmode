---
name: "godmode:task-driven-development"
description: >
  Use when implementing any feature or bugfix, before writing implementation code. Triggers
  on "implement", "add feature", "fix bug", "write code for", or any task that produces
  production Rust code. Structures red/green/refactor as serialized YAML task entries with
  explicit sequential issue chains and phase state.
---

# Task-Driven Development

**Iron Law**: No production code without a prior failing test. Phases are serialized in a YAML
task list — the file is the ground truth. Move tasks forward; never skip phases.

## Task List Schema

Each unit of work is a task entry in `tdd-tasks.yaml` (at the repo root or `.ctx/`):

```yaml
# tdd-tasks.yaml
tasks:
  - id: t1
    title: "parser rejects empty input"
    crate: godmode-core
    test: "parser_empty_input_returns_err"
    phase: red # red | green | refactor | done
    status: active # pending | active | done | failed
    depends_on: [] # sequential chain: this task blocks t2

  - id: t2
    title: "parser accepts valid identifier"
    crate: godmode-core
    test: "parser_valid_ident_returns_ok"
    phase: pending
    status: pending
    depends_on: [t1] # won't start until t1.status == done

  - id: t3
    title: "parser round-trips through serializer"
    crate: godmode-core
    test: "parser_roundtrip_matches_input"
    phase: pending
    status: pending
    depends_on: [t2]
```

See `helpers/task-schema.yaml` for the annotated schema with all fields.

## Workflow

### 0. Build the task list first

Before touching any source file, write out every task you intend to implement. Each task maps
to exactly one test. Group related tasks into sequential chains using `depends_on`.

```bash
# Bootstrap a task file for the current work unit
rust-script skills/task-driven-development/helpers/task-runner.rs init "feat: parser" --crate godmode-core
```

Or write `tdd-tasks.yaml` manually using the schema above.

### 1. RED — write a failing test

Advance the next `pending` task to `phase: red, status: active`:

```bash
rust-script skills/task-driven-development/helpers/task-runner.rs red t1
```

Write the test. Run it and confirm it fails for the right reason — not a compile error:

```bash
cargo nextest run -p <crate> -E 'test(<test_name>)'
```

The runner sets `phase: red` and records `started_at` in the task file.

### 2. GREEN — write minimum implementation

Write the least code to make the test pass. Then advance:

```bash
rust-script skills/task-driven-development/helpers/task-runner.rs green t1
```

This runs `cargo nextest run -p <crate>` and only advances the task if all tests pass.
On success: `phase: green`.

### 3. REFACTOR — clean up with tests green

```bash
rust-script skills/task-driven-development/helpers/task-runner.rs refactor t1
```

Runs clippy + fmt + nextest. On success: `phase: done, status: done`, `completed_at` recorded.
The next task in the chain (tasks with `depends_on: [t1]`) becomes eligible.

### 4. Advance the chain

```bash
rust-script skills/task-driven-development/helpers/task-runner.rs next
# prints the next eligible task ID
```

Repeat from step 1 for each task in the list.

## 3-Attempt Rule

If a test is still failing after 3 red→green attempts, mark the task `status: failed` and stop:

```bash
rust-script skills/task-driven-development/helpers/task-runner.rs fail t1 --reason "architecture needs redesign"
```

Then either redesign or ask the user. Never brute-force past 3 failures.

## Rust-Specific Rules

- Tests live in `#[cfg(test)]` modules in the same file, or `tests/` for integration tests.
- Use `cargo nextest run -p <crate> -E 'test(<name>)'` to run a single test.
- `RUST_BACKTRACE=1 cargo nextest run -p <crate>` for panics.
- Trait doubles (in-memory fakes) over mocked HTTP/DB in unit tests.
- Domain code has zero imports from infrastructure crates.
- New external dependencies go behind a trait (port) — implementations in adapters.

## Issue Chain Integration

When tasks correspond to GitHub or Linear issues, add `issue:` to each entry:

```yaml
- id: t1
  title: "parser rejects empty input"
  issue: "gh:42" # gh:<number> or linear:<id>
  crate: godmode-core
  test: "parser_empty_input_returns_err"
  phase: red
  status: active
  depends_on: []
```

Close issues as tasks reach `status: done`:

```bash
rust-script skills/task-driven-development/helpers/task-runner.rs close-issues
# closes all gh: issues whose task.status == done
```

## Commit Discipline

Commit after each `refactor` phase completes — one commit per green+clean task:

```bash
git branch --show-current   # verify branch before every commit
git commit -m "feat(<crate>): <task title>"
```

Never commit across multiple tasks in one shot. The task file documents what was done and when.

## Invalid Rationalizations

| Thought                                              | Truth                                                 |
| ---------------------------------------------------- | ----------------------------------------------------- |
| "Too simple to test"                                 | Simple code breaks too                                |
| "I'll add it to the task list after"                 | Tasks written after pass immediately — proves nothing |
| "I manually tested it"                               | Not repeatable, not systematic                        |
| "Just keeping code as reference while writing tests" | You will adapt it — that's test-after                 |
| "I'll do t3 before t2 is done"                       | Respect the chain — depends_on is a contract          |

## Additional Resources

- **`helpers/task-schema.yaml`** — annotated schema with all fields and valid values
- **`helpers/task-runner.rs`** — CLI for advancing task phase state
- **`references/test-patterns.md`** — unit/integration/async test patterns, naming conventions
- **`references/test-stub.rs.template`** — copy-paste starting point for a new test module
