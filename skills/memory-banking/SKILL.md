---
name: "godmode:memory-banking"
description: >
  Generate and maintain a `.ctx/memory-bank/` directory that captures project context backed
  by source code. The memory bank is injected into prompts via lifecycle hooks and updated
  incrementally as work progresses. Use when starting a new project, onboarding to an unfamiliar
  codebase, or when context drift is detected.
requires: []
next: []
---

# Memory Banking

A persistent, source-backed context layer that lives at `.ctx/memory-bank/` in any project.
It captures what the project is, how it works, what's done, and what's active — grounded in
actual code, not assumptions.

## When to use

- First session in a new or unfamiliar project (no `.ctx/memory-bank/` exists)
- After major architectural changes that invalidate existing context
- When the SessionStart hook reports stale memory-bank files
- Explicitly via `/memory-bank` or "update the memory bank"

## Directory structure

```
.ctx/memory-bank/
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

3. **Write to `.ctx/memory-bank/`** — create the directory if absent. Overwrite existing files
   only when regenerating; for incremental updates, use the Edit tool on individual files.

## Update rules

- **active-context.md** — update at every significant task transition (start, block, complete)
- **progress.md** — update when features are completed or new issues surface
- **tech-context.md** — update when deps change or build commands are modified
- **system-patterns.md** — update when architecture decisions are made
- **project-brief.md / product-context.md** — rarely change; update only on scope shifts

## Conversation knowledge capture (`remember`)

Use when the user explicitly asks to "remember this", "save what we learned", "update memory",
or asks for "capturing learnings."

1. Re-scan the current conversation and list candidate learnings.
2. Classify each item into one of these buckets:
   1. **Best practices** — patterns that worked, anti-patterns to avoid, and decision rationale.
   2. **Quality standards** — criteria for good code, docs, processes, or outputs.
   3. **Operational knowledge** — architecture decisions, workflows, tools, and reusable conventions.
3. For each best practice or decision, identify where to store it:
   1. **Global memory** (`~/.deepagents/agent/AGENTS.md`) for universal preferences.
   2. **Project memory** (`.deepagents/AGENTS.md`) for repo-specific rules.
   3. **Reusable skill** (`~/.deepagents/agent/skills/<name>/SKILL.md`) for multi-step methods.
4. Capture reusable workflows as skills when they include repeated methodology.
5. Add simple preferences to AGENTS files with one or two concrete bullet rules.
6. Keep records minimal and non-obvious; avoid dumping conversational noise.

### Memory storage decision rules

- If a behavior is a one-off preference, store it as a memory bullet.
- If a process is repeatable and decision-heavy, store it as a dedicated skill.
- Never store unverified claims; prefer concrete paths, commands, and evidence from files.

### Memory-first file convention

- Prefer project file updates for preferences that depend on this repository.
- Use global memory only when the rule applies across projects.
- Prefer skill updates when behavior requires multiple ordered steps or quality gates.

### Quick validation

Before handing off or committing `memory-banking`, run:

- `rust-script skills/memory-banking/helpers/quick_validate.rs skills/memory-banking/SKILL.md`

This Rust-script validator checks:

- frontmatter starts and closes correctly;
- required keys (`name`, `description`) exist;
- only approved frontmatter keys are present;
- empty, long, or malformed fields are rejected.

For stronger checks, run:

`agentlint --difficulty hard skills/memory-banking/SKILL.md`

This uses `plugins/agentlint.memory-banking.toml` if the custom plugin is present.

## Staleness detection

The SessionStart hook checks file modification times. If any memory-bank file is older than
the most recent 5 commits, it prints a staleness warning. The agent should then read the
stale file(s) and update them based on recent changes.

## Integration with other skills

- `godmode:context-map` can feed into `system-patterns.md`
- `godmode:task-driven-development` transitions update `active-context.md`
- `godmode:verification-before-completion` should check memory-banking is current before shipping
