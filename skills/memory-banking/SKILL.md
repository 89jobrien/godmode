---
name: "godmode:memory-banking"
description: >
  Generate and maintain a `.ctx/memory-banking/` directory that captures project context backed
  by source code. The memory bank is injected into prompts via lifecycle hooks and updated
  incrementally as work progresses. Use when starting a new project, onboarding to an unfamiliar
  codebase, or when context drift is detected.
---

# Memory Banking

A persistent, source-backed context layer that lives at `.ctx/memory-banking/` in any project.
It captures what the project is, how it works, what's done, and what's active — grounded in
actual code, not assumptions.

## When to use

- First session in a new or unfamiliar project (no `.ctx/memory-banking/` exists)
- After major architectural changes that invalidate existing context
- When the SessionStart hook reports stale memory-bank files
- Explicitly via `/memory-bank` or "update the memory bank"

## Directory structure

```
.ctx/memory-banking/
  project-brief.md      # what, who, done-criteria
  product-context.md    # why it exists, UX principles
  tech-context.md       # stack, deps, build commands, constraints
  system-patterns.md    # architecture, data flow, conventions
  active-context.md     # current focus, in-progress, decisions, questions
  progress.md           # what works, what's in progress, what's not started
```

## Generation procedure

When creating or fully regenerating the memory bank:

1. **Read sources** — scan these files (skip missing ones silently):
   - `README.md`, `CLAUDE.md`, `.claude.local.md`
   - `Cargo.toml` / `package.json` / `pyproject.toml` (root + workspace members)
   - `src/lib.rs` or `src/main.rs` (top-level module docs and pub API)
   - `.github/workflows/*.yml` (CI structure)
   - `CHANGELOG.md`, `HISTORY.md`
   - `.ctx/HANDOFF.*.yaml` (recent session state)
   - `git log --oneline -20` (recent commit history)

2. **Populate each file** using the templates in `references/`. Every claim must trace to a
   specific file path or git SHA. Do not invent features or state that isn't evidenced in code.

3. **Write to `.ctx/memory-banking/`** — create the directory if absent. Overwrite existing files
   only when regenerating; for incremental updates, use the Edit tool on individual files.

## Update rules

- **active-context.md** — update at every significant task transition (start, block, complete)
- **progress.md** — update when features are completed or new issues surface
- **tech-context.md** — update when deps change or build commands are modified
- **system-patterns.md** — update when architecture decisions are made
- **project-brief.md / product-context.md** — rarely change; update only on scope shifts

## Staleness detection

The SessionStart hook checks file modification times. If any memory-bank file is older than
the most recent 5 commits, it prints a staleness warning. The agent should then read the
stale file(s) and update them based on recent changes.

## Integration with other skills

- `godmode:context-map` can feed into `system-patterns.md`
- `godmode:task-driven-development` transitions update `active-context.md`
- `godmode:verification-before-completion` should check memory-banking is current before shipping
