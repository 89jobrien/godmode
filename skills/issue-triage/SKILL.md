---
name: "godmode:issue-triage"
description: >
  Triage and prioritize GitHub issues by type (bug/feature/chore), complexity
  (S/M/L), and priority (P1-P4). Detect duplicates, propose execution order,
  and generate task graphs for godmode automation.
requires: []
next: [task-management]
---

# Issue Triage

Systematically analyze open GitHub issues to establish priority, complexity,
and task dependencies. Use before sprint planning or when the backlog is
unclear.

## When to Use

- At the start of a development cycle to plan work
- After a release to sort incoming bugs and feature requests
- To consolidate duplicate issues and establish order
- When asked to "triage issues", "prioritize issues", or "what needs doing"

## Classification Rules

### Type Classification

| Type    | Indicators                                                 | Priority Uplift |
| ------- | ---------------------------------------------------------- | --------------- |
| Bug     | Error, crash, panic, "doesn't work", regression, breakage  | +1 level        |
| Feature | "Add support for", enhancement, new capability, API change | none            |
| Chore   | Docs, cleanup, refactoring, dependency bump, test coverage | none            |

**Decision tree**:

1. Is there a _problem_ being reported? → Bug
2. Is a _new capability_ being requested? → Feature
3. Is this _housekeeping_ (docs, tests, deps)? → Chore
4. If mixed (e.g., "fix bug and add test"), classify by primary intent

### Complexity Estimation

| Size | Characteristics                                     | Example                                 |
| ---- | --------------------------------------------------- | --------------------------------------- |
| S    | Single file, obvious fix, no design debate          | Fix typo in README, bump version        |
| M    | Multi-file, some design, cross-module impact        | Add a new CLI flag, refactor one module |
| L    | Cross-crate, needs plan, requires RFC or design doc | New agent system, major API overhaul    |

**Time proxy**: S ≈ 1–2 hours, M ≈ 1–2 days, L ≈ 1–2 weeks (before planning).

### Priority Framework

| Priority | Criteria                                                        | Action         |
| -------- | --------------------------------------------------------------- | -------------- |
| P1       | Blocking (security, outage, data loss, high-rep user)           | Start next     |
| P2       | Important (frequent report, significant feature, team blockers) | Sprint in      |
| P3       | Nice-to-have (incremental improvement, niche use case)          | Backlog        |
| P4       | Speculative (vague request, "someday", personal preference)     | Consider later |

**Application**: Assign base priority from table, then apply type uplift (bugs
+1 level), then consider user urgency/reputation.

## Duplicate Detection Heuristics

Check for:

- **Exact phrase matches** in titles or first line of body (e.g., "panic on
  invalid UTF-8")
- **Component + action pairs** (e.g., both mention "parser" + "crash")
- **Error messages** (e.g., two issues cite the same stack trace)
- **Feature equivalence** (e.g., "add YAML support" vs. "support YAML files")
- **Related issues** linked in comments (check body for `#<number>` refs)

**Handling**: Flag as related, group in triage output, recommend consolidation
to user but do not merge without approval.

## Task Graph Generation

When converting triage to godmode tasks:

1. **Map issue → task**: Each issue becomes one task (or splits if
   multi-phase).
2. **Assign IDs**: Use sequential (`t1`, `t2`, ...) or issue-number-based
   (`t42`, `t51`, ...).
3. **Build deps**:
   - P1 blocker → no deps (root)
   - Bug fix → no deps (unblock others)
   - Feature → depend on related bugs or foundational tasks
   - Chore → usually no deps, or after related feature
4. **Specify crate name** if applicable.
5. **Set run: field** if the issue describes a test or script.

**Example**:

```bash
# Bug (P1) — no deps
godmode task add "Fix panic on invalid UTF-8 input" \
  --id t1 --crate-name parser --priority high

# Related feature — depends on bug fix
godmode task add "Add YAML support" \
  --id t2 --depends-on t1 --crate-name parser

# Independent chore
godmode task add "Update README with examples" \
  --id t3 --priority normal
```

## Integration with godmode Workflow

### Pull Issues into Tasks

```bash
# Fetch open issues and convert to tasks manually:
gh issue list --repo <owner/repo> --state open \
  --json number,title,labels --limit 50
# Then add each as a task:
godmode task add "<title>" --id t<N> --crate-name <crate>
```

> **Note**: `godmode task pull --github [--repo owner/repo] [--label <label>]` imports
> open GitHub issues as tasks. Or use `gh issue list` and add tasks individually.

### View Task Graph After Triage

```bash
godmode task list              # show all tasks
godmode status                 # show ready and runnable
godmode dispatch               # emit parallel chains JSON
```

## Triage Output Format

Produce a table with these columns:

```
| # | Title | Type | Size | Priority | Group | Order | Notes |
|---|-------|------|------|----------|-------|-------|-------|
```

Where:

- `#`: GitHub issue number
- `Title`: Issue title (truncated to ~40 chars)
- `Type`: Bug, Feature, or Chore
- `Size`: S, M, or L
- `Priority`: P1, P2, P3, or P4
- `Group`: Component or feature area (e.g., "Parser", "CLI", "Docs")
- `Order`: Suggested execution rank (1 = first)
- `Notes`: Duplicates, blockers, user reputation, or other context

**Example**:

```
| 42  | Panic on UTF-8 input      | Bug     | S | P2 | Encoding    | 1 | User: @alice |
| 51  | Add YAML support          | Feature | M | P3 | Parser      | 3 | Deferred     |
| 63  | Update docs               | Chore   | S | P3 | Docs        | 4 |              |
| 48  | Same as #42 — UTF-8 panic | Bug     | S | P2 | Encoding    | — | Duplicate    |
```

## Guardrails

- **Never close or reassign issues** without explicit user approval.
- **Never modify labels** without asking first.
- **Duplicate consolidation requires user consent** — flag and recommend,
  but let the user decide.
- **Triage is advisory** — user makes final priority calls.
- **Auth check**: Verify `gh auth status` before running; fail gracefully if
  repo is private or unreachable.
- **Rate limiting**: Fetch in batches if >100 issues; cache results if
  re-running quickly.

## Example Workflow

```bash
# User: "Triage the issues in my repo"

# Agent runs:
gh issue list --repo <owner/repo> --state open \
  --json number,title,body,labels,createdAt --limit 50

# Agent classifies each (manual inspection of title/body for type/complexity)

# Agent produces triage table

# User asks: "Create tasks for P1 and P2 issues"

# Agent runs:
for each P1/P2 issue:
  godmode task add "<title>" --id t<N> --crate-name <crate> --priority high

# User runs:
godmode status          # see new tasks
godmode dispatch        # see parallel chains
```
