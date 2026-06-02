# Plan: `godmode:build-graph` Skill

**Status**: done

## Goal

A skill that guides humans through constructing and evolving a task graph over a session.
Fills the gap between `brainstorm`/`writing-plans` (design) and `task-management`
(execution loop). Usable before a plan exists (discovery mode) or after (translation mode).

## Architecture

**Crates affected:** none — skills-only, no Rust changes.

**New files:**

| File                                              | Role                                                 |
| ------------------------------------------------- | ---------------------------------------------------- |
| `skills/build-graph/SKILL.md`                     | Primary skill — phase skeleton + conversational flow |
| `skills/build-graph/helpers/graph-audit.md`       | Checklist for auditing an existing graph shape       |
| `skills/build-graph/references/graph-patterns.md` | Common graph shapes and when to use them             |

## Skill Design

### Trigger conditions

- "build a task graph", "set up my tasks", "plan out the graph"
- At session start when `.ctx/GODMODE.tasks.yaml` is absent or empty
- After `godmode:writing-plans` produces a plan doc
- Mid-session when the graph has drifted (tasks added ad hoc, deps unclear)

### Three-phase structure (from approach A)

Each phase uses conversational discovery (approach B) — one question at a time,
each answer maps to a concrete `godmode task add` or `godmode task` state call.

#### Phase 1: Shape — what tasks exist and why

Conversational prompts:

- "What's the next thing that needs to happen?"
- "Is this one unit of work or should it split into smaller steps?"
- "Which crate or module does this belong to?"

Each answer → `godmode task add <id> "<title>" [--crate-name <crate>]`

Exit condition: user says "that's everything" or no more work items surface.

#### Phase 2: Wire — dependencies between tasks

For each task added in Phase 1:

- "Does anything need to be done before this?"
- "Can this run in parallel with anything else?"

Each answer → `godmode task add` with `--depends-on` or restructure existing tasks.

After wiring: run `godmode dispatch --critical-path` and show the result.
If critical path depth is surprising (too long or too short) — surface it and ask.

#### Phase 3: Validate — graph is sound

Run and interpret:

```bash
godmode status
godmode dispatch --critical-path
godmode task next
```

Check for:

- **Orphaned tasks** — pending tasks with all deps done but not in `next` (blocked or misfiled)
- **Over-wide graph** — more than 5 independent roots (consider grouping or sequencing)
- **Single critical path** — everything sequential (consider parallelizing)
- **No runnable tasks** — all pending blocked by unfinished deps (cycle or missing task)

Each finding → one targeted question → one fix.

Exit condition: `godmode task next` returns at least one runnable task, or graph is
intentionally empty (session end).

### Evolution mode (mid-session)

When a graph already exists, skip Phase 1 discovery and open with Phase 3 audit:

1. Read current graph with `godmode status`
2. Identify the most urgent structural issue (blocked tasks, stale running tasks, wrong shape)
3. Fix it with one targeted edit
4. Re-run `godmode status` — repeat until shape is sound

### Integration with other skills

| Skill                     | Relationship                                               |
| ------------------------- | ---------------------------------------------------------- |
| `godmode:writing-plans`   | `build-graph` translates an existing plan into a graph     |
| `godmode:task-management` | `build-graph` hands off to `task-management` after Phase 3 |
| `godmode:parallel-agents` | `build-graph` output (chains) feeds directly into dispatch |
| `godmode:brainstorm`      | `brainstorm` precedes `build-graph` for new features       |

## Tech Decisions

- **One question at a time** — never dump a list of questions. Mirrors `brainstorm` discipline.
- **CLI-first** — every phase step maps to a concrete `godmode` CLI call shown to the user.
- **Non-destructive by default** — skill never removes or modifies existing tasks without
  explicit user confirmation. Additions only unless user asks to restructure.
- **Phase exit is flexible** — user can skip any phase or exit early. The skill adapts.

## Out of Scope

- Automatic graph generation from code analysis
- Visualizing the graph as a diagram (that's a separate TUI/HTML concern)
- Merging or diffing two graphs
- Suggesting task titles from git history or issues

---

## Tasks

### Task 1: Write `skills/build-graph/SKILL.md`

**File**: `skills/build-graph/SKILL.md`
**Run**: `just conformance`

Write the full skill following the three-phase structure. Frontmatter:

```yaml
---
name: "godmode:build-graph"
description: >
  Use when constructing or evolving a task graph — at session start, after a plan is
  written, or mid-session when the graph has drifted. Guides through shape, wire, and
  validate phases one question at a time.
---
```

Each phase section must include:

- The conversational prompt(s) to ask the user
- The exact `godmode` CLI command(s) that result from each answer
- Exit condition for the phase

### Task 2: Write helper and reference files

**Files**:

- `skills/build-graph/helpers/graph-audit.md`
- `skills/build-graph/references/graph-patterns.md`

`graph-audit.md`: before/during/after checklist for auditing graph shape. Covers orphans,
over-wide, single critical path, no runnable tasks.

`graph-patterns.md`: named graph shapes with when to use each:

- Linear chain (strict sequential work)
- Parallel fan-out (independent crates)
- Diamond (shared dep, parallel middle, join)
- Mixed (realistic multi-crate session)

### Task 3: Add to skill-index and conformance

**Files**:

- `skills/using-godmode/references/skill-index.md`

Add `godmode:build-graph` row to the trigger table. Add it to the skill chain diagram
between `writing-plans` and `task-management`.

Run `just conformance` — all checks green.

Commit: `feat(skills): add godmode:build-graph skill`
