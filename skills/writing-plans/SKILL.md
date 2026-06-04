---
name: "godmode:writing-plans"
description: >
  Use when you have a spec, approved design, or requirements for a multi-step task, before
  touching any code. Triggers after brainstorming completes, or when given a feature spec
  to implement.
---

# Writing Plans

A plan is a complete implementation guide that a fresh agent with no prior context could
execute correctly. If it requires context not in the document, it is incomplete.

## Before Writing: Context Map

Run `godmode:context-map` first. A plan written without a context map will have incomplete
task lists, missed consumers, and wrong crate assignments.

The context map output populates:

- **Architecture → Crates affected** — from the map's Files to Modify table
- **Architecture → Data flow** — from dependency edges
- **Each task's `**File(s)**:`** — exact paths from the map, not guesses
- **Risk section** — copied directly from the map's Risk checklist

Do not start writing tasks until the map is complete and reviewed.

---

## Plan Structure

Save to: `docs/plans/YYYY-MM-DD-<feature-name>.md`

````markdown
# Plan: <Feature Name>

## Goal

One sentence. What does this implement and why.

## Architecture

- Crates affected: list them
- New traits/types: name and location
- Data flow: source → transform → sink

## Tech Stack

- Rust edition, key crates used
- Any new dependencies and why

## Tasks

### Task 1: <name>

**Crate**: `<crate-name>`
**File(s)**: `crates/<crate>/src/<file>.rs`
**Run**: `cargo nextest run -p <crate>`

The optional `**Run**:` annotation attaches a shell command to the task. When present,
`godmode task run <id>` executes it. Use `--auto-done` to automatically mark the task done
on exit 0: `godmode task run t1 --auto-done`.

Prefix with `rx:` to invoke a script from the rx registry: `**Run**: rx:my-script`

Commands containing shell metacharacters (`>`, `|`, `&`, `;`) are automatically run via
`sh -c`, so redirects and pipes work as expected:
`**Run**: cargo test 2>&1 | tee /tmp/results.txt`

1. Write failing test:
   ```rust
   #[test]
   fn test_<name>() { ... }
   ```
````

Run: `cargo nextest run -p <crate> -- test_<name>`
Expected: FAIL

2. Implement:

   ```rust
   // exact code here — no placeholders
   ```

3. Verify:

   ```
   cargo nextest run -p <crate>    → all green
   cargo clippy -p <crate> -- -D warnings  → zero warnings
   ```

4. Run: `git branch --show-current`
   Verify output matches the expected branch. Stop immediately if not.
   Commit: `git commit -m "feat(<crate>): <summary>"`

### Task 2: ...

```

## Quality Rules

- **No placeholders**: never write "TBD", "similar to Task N", "add error handling here"
- **Exact paths**: every file path must be complete and correct
- **Exact code**: every code block must be copy-paste ready
- **Consistent names**: types and method names must match across all tasks
- **TDD for every task**: write failing test → verify failure → implement → verify pass → commit

## Pre-Save Checklist

Check these before writing the file:
- [ ] Every requirement maps to at least one task
- [ ] No placeholders or vague directives anywhere
- [ ] Method names and types are consistent across all tasks
- [ ] Each task is 2-5 minutes of focused work (if longer, split it)
- [ ] Each task ends with a commit

## After Writing

Update `.ctx/godmode/tasks.yaml` with a task entry for each plan task.
See `godmode:task-management` for the schema.

## Additional Resources

- **`references/plan-format.md`** — full format spec, `**Run**:` annotation table, dep model
- **`helpers/plan-template.md`** — blank plan to fill in
```
