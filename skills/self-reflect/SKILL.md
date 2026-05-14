---
name: "godmode:self-reflect"
description: >
  Session reflection — review what was accomplished this session, surface patterns and
  surprises, and write a structured retrospective. Use when asked to "reflect", "self-reflect",
  "what did we do this session", or at natural session close points.
---

# Self-Reflect

Review the current session's work and produce a structured retrospective written to
`.ctx/reflect-<YYYY-MM-DD>.md`.

## When to Use

- At the end of a working session before handoff
- When asked to "reflect" or "what did we do today"
- After a significant batch of commits lands (e.g. post-wave integration)
- Before cutting a release to sanity-check what actually changed

## Step 1: Collect session evidence

Gather the raw material for the reflection:

```bash
# Commits since session start (last 24h is a safe proxy; adjust if session spans days)
git log --oneline --since="24 hours ago"

# Files changed across those commits
git diff --stat HEAD~1 HEAD
```

Use the Glob tool to list any files in the working scratch dir:

```
Glob: pattern=".ctx/_WORKING_DIR/*"
```

Current task graph state — use `godmode task list`:

```bash
godmode task list
```

Outstanding blocked tasks:

```bash
godmode task list --json | rg blocked
```

Also read `.ctx/HANDOFF.*.yaml` if present — it captures intent from session start.

## Step 2: Identify what was accomplished

From the commit log and diff stat, extract:

- **Shipped**: concrete changes that landed (commits, features, fixes, TODOs added)
- **Unfinished**: tasks still running or pending, files modified but not committed
- **Discarded / pivoted**: things started but abandoned (check `git stash list`)

## Step 3: Surface patterns and surprises

Answer these questions honestly based on the evidence:

1. **What took longer than expected?** — look for repeated edits to the same file, or
   multiple fix-up commits on the same topic.
2. **What went smoothly?** — first-pass successes, clean test runs, no hook failures.
3. **What was discovered that wasn't in the original plan?** — TODOs added, unexpected
   coupling found, agents that surfaced issues not on the task graph.
4. **What would speed up the next session?** — missing tools, unclear specs, slow feedback
   loops, friction points.

## Step 4: Write the retrospective

Write to `.ctx/reflect-<YYYY-MM-DD>.md` (overwrite if exists for today):

```markdown
# Session Reflection — <YYYY-MM-DD HH:MM>

## Shipped

- <bullet per commit or logical change group>

## Unfinished

- <pending tasks, uncommitted changes, stashed work>
  - None if graph is clean and worktree is clean.

## Patterns & Surprises

### Took longer than expected

- <finding or "nothing notable">

### Went smoothly

- <finding>

### Discovered mid-session

- <unexpected finding, new TODO, coupling, etc.>

### Next session speedups

- <concrete suggestion — a missing test, a flaky tool, an unclear spec>

## Task graph snapshot

<paste output of `godmode task list`>

## Open questions

- <anything unresolved that the next session should address first>
```

Use the Write tool to create the file. Print a short summary (Shipped / Unfinished /
Top insight) to stdout as well.

## Step 5: Update task graph (if applicable)

If any tasks were completed during the session but not marked done:

```bash
godmode task done <id> --commit <sha> --notes "<brief>"
```

If new work was discovered that belongs on the graph:

```bash
godmode task add "<title>" --crate-name <crate>
```

Do not invent tasks to fill the graph — only add what was genuinely discovered.

## Guardrails

- Do not modify source code during reflection — read-only except for the `.ctx/` report
  and task graph state updates.
- Do not rewrite history or amend commits to make the session look cleaner than it was.
- If the session had no commits, say so — an empty "Shipped" section is honest and useful.
- The reflection is for the next session's benefit, not performance review. Be precise,
  not flattering.
- **Never write `(fill in next session)` or any deferred placeholder.** Every section must
  contain actual content from this session. If a section genuinely has nothing to report,
  write `- Nothing notable.` — not a deferral. Deferred placeholders make the reflection
  useless to the next agent.
