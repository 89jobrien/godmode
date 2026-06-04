---
name: "godmode:using-godmode"
description: >
  Use at the start of every conversation to orient to available skills and workspace rules.
  Replaces superpowers entirely. Triggers on session start, "/godmode", or "what skills do
  you have".
requires: []
next: []
---

You have godmode — a self-contained Rust-native development methodology.

## Instruction Priority

1. User's explicit instructions (CLAUDE.md, direct requests) — highest
2. Godmode skills — override defaults
3. Default system prompt — lowest

## Phased Workflow

Every non-trivial task progresses through five phases. Short
confirmations ("do it", "act", "go") advance to the next phase.

<godmode-phase name="ORIENT" mode="read-only" response-header="# Phase: ORIENT" skills="godmode handon">
Default phase. Read files, run `godmode handon`, summarize state. No modifications.
</godmode-phase>

<godmode-phase name="PLAN" mode="read-only" response-header="# Phase: PLAN" skills="brainstorm, writing-plans">
Produce a written plan. Still read-only. End with "Type ACT to proceed."
</godmode-phase>

<godmode-phase name="ACT" mode="read-write" response-header="# Phase: ACT" skills="task-driven-development, parallel-agents">
Edit files, run commands, dispatch subagents. Enter on user approval.
</godmode-phase>

<godmode-phase name="VERIFY" mode="read + test" response-header="# Phase: VERIFY" skills="verification-before-completion">
Run tests and quality gates. Return to ACT if failures. Ask to SHIP when green.
</godmode-phase>

<godmode-phase name="SHIP" mode="commit/push" response-header="# Phase: SHIP" skills="cap, handoff">
Commit, push, handoff. Only on explicit approval. Return to ORIENT after.
</godmode-phase>

**Phase transitions**: Users can skip phases ("just fix it" = all phases in one pass).
After each ACT turn, default to VERIFY. Multiple ACT turns are fine.

## Skill Invocation Rule

Before responding in any phase, check if a skill applies.
1% chance it's relevant = invoke it.

```
Message received → skill applies? → YES: invoke Skill tool first → NO: respond
```

**Priority**: Process skills first (`brainstorm`, `systematic-debugging`),
then implementation skills (`task-driven-development`, `parallel-agents`),
then quality gates (`verification-before-completion`, `code-review`).

## Red Flags (you are rationalizing — stop)

- "This is simple" → check anyway
- "I need context first" → skill check comes before exploration
- "The skill is overkill" → if it exists, use it
- "Just this one thing first" → check before acting

## Available Skills

| Skill                                     | When                                               |
| ----------------------------------------- | -------------------------------------------------- |
| `godmode:task-driven-development`         | TDD with serialized task lists and issue chains    |
| `godmode:systematic-debugging`            | Any bug, test failure, unexpected behavior         |
| `godmode:brainstorm`                      | Before any creative or design work                 |
| `godmode:writing-plans`                   | Multi-step task with a spec or requirements        |
| `godmode:verification-before-completion`  | Before claiming work is done                       |
| `godmode:task-management`                 | Creating, tracking, or executing a task graph      |
| `godmode:parallel-agents`                 | 2+ independent tasks that can run concurrently     |
| `godmode:cap`                             | "cap", "commit and push", "ship it"                |
| `godmode:ci-fix`                          | CI failing, "fix CI", broken pipeline              |
| `godmode:tackle-issues`                   | Working on GitHub issues in parallel               |
| `godmode:code-review`                     | Before merge, after feature complete               |
| `godmode:refactoring`                     | Restructuring code without changing behaviour      |
| `godmode:receiving-review`                | Processing incoming review comments                |
| `godmode:observability-as-infrastructure` | Adding tracing to helpers and subagents            |
| `godmode:testing-philosophy`              | Designing test strategy for new code               |
| `godmode:introspection`                   | Auditing skills for consistency after changes      |
| `godmode:moa`                             | Multi-model synthesis via Mixture of Agents        |
| `godmode:todo-issue-sync`                 | Auditing inline TODOs against tracked issues       |
| `godmode:self-reflect`                    | End-of-session retrospective — "reflect"           |
| `godmode:wave-integration`                | Merge parallel agent branches sequentially         |
| `godmode:merge`                           | Merge branch, create PR, squash, worktree cleanup  |
| `godmode:rust-conventions`                | Rust coding conventions for writing/reviewing code |
| `godmode:context-map`                     | Map all relevant files before implementation       |
| `godmode:decompose`                       | Split large diff/PR into smaller branches          |
| `godmode:doublecheck`                     | Three-layer verification of factual claims         |
| `godmode:mini-context-graph`              | Persistent knowledge base with entity graph        |
| `godmode:agent-governance`                | Governance/safety patterns for AI agent systems    |
| `godmode:memory-banking`                  | Generate/maintain .ctx/memory-bank/ context        |
| `godmode:changelog`                       | Parse git history into structured changelogs       |
| `godmode:cross-issue`                     | Cross-repo issue coordination and linking          |
| `godmode:dead-code`                       | Find unused public API, orphaned tests, stale refs |
| `godmode:dep-audit`                       | Audit deps via cargo outdated/deny/audit           |
| `godmode:dep-bump`                        | Propagate workspace crate version bumps downstream |
| `godmode:doc-maintainer`                  | Audit docs against source code for drift           |
| `godmode:health-score`                    | Measure codebase health across seven metrics       |
| `godmode:issue-triage`                    | Triage and prioritize GitHub issues                |
| `godmode:mistake-tracker`                 | Catalog recurring mistakes and failure modes       |
| `godmode:pattern-learner`                 | Cross-session pattern extraction from traces       |
| `godmode:pr-author`                       | Compose PR descriptions from branch context        |
| `godmode:release-notes`                   | Write user-facing release notes from git history   |
| `godmode:workspace-refactor`              | Catalog breaking changes in shared crate APIs      |

## Always-Active Rules

- **Never `--no-verify`** on commits. Pre-commit hooks always run.
- **Commit signing**: SSH key via 1Password agent. Unlock 1Password and retry on failure.
- **Shell**: Nushell (`nu`) primary. No bash-isms (`&&`, `$()`, `export VAR=`) in `.nu` files.
- **Git in other repos**: `git -C <path>` not `cd <path> && git`.
- **Rust**: `cargo check` + `cargo clippy` after every change. `cargo nextest` over `cargo test`.
  Fix all clippy warnings before committing.
- **Secrets**: Never pass raw `op://` URIs. Use `op read` or `op run`.
- **Scope**: Touch only files within the explicitly requested scope.
- **Destructive ops**: Show diff, get explicit confirmation before dropping stashes or
  deleting files.

## Session Ritual

**Start:**

```bash
godmode handon      # triage: running tasks, next runnable, next doob todo
```

**End:**

```bash
godmode handoff     # warns on leaked running tasks, writes hj handoff
```

Or with reflection:

```bash
godmode handoff && godmode:self-reflect   # handoff + structured retrospective
```

## Additional Resources

- **`references/skill-index.json`** — structured skill registry with triggers and phase mappings
- **`helpers/session-start.sh`** — run at session start to verify CLI and triage

## CLI Quick Reference

```bash
godmode handon                          # session start triage
godmode handoff                         # session end closeout
godmode plan ingest <plan.md>           # ingest plan → task graph
godmode task list [--json]              # all tasks
godmode task next [--json]              # next runnable (exit 1 if none)
godmode task add <id> "<title>" [opts]  # add task
godmode task start <id>                 # mark running
godmode task done <id> [--commit <sha>] # mark done
godmode task block <id> "<reason>"      # mark blocked
godmode task unblock <id>               # reset to pending
godmode task unblock-all                # reset ALL blocked tasks to pending
godmode task run <id>                   # run task's run: command
godmode dispatch [--max 5] [--json]     # parallel chains for orca-strait
godmode agent dispatch <path> [--max N] # ingest + dispatch in one step
```
