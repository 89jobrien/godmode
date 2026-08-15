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

<godmode-phase name="SHIP" mode="commit/push" response-header="# Phase: SHIP" skills="cap, session-wrap-commit-push">
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

| Skill                                     | When                                                         |
| ----------------------------------------- | ------------------------------------------------------------ |
| `godmode:task-driven-development`         | TDD with serialized task lists and issue chains              |
| `godmode:systematic-debugging`            | Any bug, test failure, unexpected behavior                   |
| `godmode:brainstorm`                      | Before any creative or design work                           |
| `godmode:writing-plans`                   | Multi-step task with a spec or requirements                  |
| `godmode:verification-before-completion`  | Before claiming work is done                                 |
| `godmode:task-management`                 | Creating, tracking, or executing a task graph                |
| `godmode:parallel-agents`                 | 2+ independent tasks that can run concurrently               |
| `godmode:cap`                             | "cap", "commit and push", "ship it"                          |
| `godmode:ci-fix`                          | CI failing, "fix CI", broken pipeline                        |
| `godmode:tackle-issues`                   | Working on GitHub issues in parallel                         |
| `godmode:code-review`                     | Before merge, after feature complete                         |
| `godmode:refactoring`                     | Restructuring code without changing behaviour                |
| `godmode:receiving-review`                | Processing incoming review comments                          |
| `godmode:observability-as-infrastructure` | Adding tracing to helpers and subagents                      |
| `godmode:testing-philosophy`              | Designing test strategy for new code                         |
| `godmode:introspection`                   | Auditing skills for consistency after changes                |
| `godmode:moa`                             | Multi-model synthesis via Mixture of Agents                  |
| `godmode:todo-issue-sync`                 | Auditing inline TODOs against tracked issues                 |
| `godmode:self-reflect`                    | End-of-session retrospective — "reflect"                     |
| `godmode:wave-integration`                | Merge parallel agent branches sequentially                   |
| `godmode:merge`                           | Merge branch, create PR, squash, worktree cleanup            |
| `godmode:rust-conventions`                | Rust coding conventions for writing/reviewing code           |
| `godmode:context-map`                     | Map all relevant files before implementation                 |
| `godmode:decompose`                       | Split large diff/PR into smaller branches                    |
| `godmode:doublecheck`                     | Three-layer verification of factual claims                   |
| `godmode:mini-context-graph`              | Persistent knowledge base with entity graph                  |
| `godmode:agent-governance`                | Governance/safety patterns for AI agent systems              |
| `godmode:memory-banking`                  | Generate/maintain .ctx/memory-bank/ context                  |
| `godmode:changelog`                       | Parse git history into structured changelogs                 |
| `godmode:cross-issue`                     | Cross-repo issue coordination and linking                    |
| `godmode:dead-code`                       | Find unused public API, orphaned tests, stale refs           |
| `godmode:dep-audit`                       | Audit deps via cargo outdated/deny/audit                     |
| `godmode:dep-bump`                        | Propagate workspace crate version bumps downstream           |
| `godmode:doc-maintainer`                  | Audit docs against source code for drift                     |
| `godmode:health-score`                    | Measure codebase health across seven metrics                 |
| `godmode:issue-triage`                    | Triage and prioritize GitHub issues                          |
| `godmode:mistake-tracker`                 | Catalog recurring mistakes and failure modes                 |
| `godmode:pattern-learner`                 | Cross-session pattern extraction from traces                 |
| `godmode:pr-author`                       | Compose PR descriptions from branch context                  |
| `godmode:release-notes`                   | Write user-facing release notes from git history             |
| `godmode:workspace-refactor`              | Catalog breaking changes in shared crate APIs                |
| `godmode:agents-skill-save`               | Create or fix a local skill saved to wrong path              |
| `godmode:baml-add-types`                  | Add BAML types/functions to cruxx-agentic                    |
| `godmode:design`                          | Translate brainstorm into architectural spec                 |
| `godmode:dual-forge-pr-merge`             | PR across GitHub and Gitea mirrors                           |
| `godmode:gh-bulk-issues`                  | Create 3+ GitHub issues with consistent format               |
| `godmode:notfiles-release-workflow`       | Release workflow for the notfiles repo                       |
| `godmode:planning-with-crux`              | Design crux DSL pipelines and macro agents                   |
| `godmode:release-readiness-check`         | Pre-release verification of tags, crates, gates              |
| `godmode:remote-upstream-triage`          | Fix git push/PR upstream or remote drift                     |
| `godmode:repo-gap-backlog`                | Turn local project gaps into GitHub issues                   |
| `godmode:rust-release-workflow-author`    | Create GitHub Actions release workflow for Rust              |
| `godmode:rustqual`                        | Rust code quality analysis via rustqual CLI                  |
| `godmode:rustqual-workspace`              | rustqual guidance for multi-crate Rust workspaces            |
| `godmode:session-wrap-commit-push`        | End-of-session commit and push closeout                      |
| `godmode:token-cost-optimizer`            | Analyze or reduce Claude/agent token costs                   |
| `godmode:using-crux`                      | Navigate, build, or extend the crux codebase                 |
| `godmode:whatidid`                        | Generate daily activity report from session data             |
| `godmode:workspace-bump-commit`           | Apply version bumps and create release commit                |
| `godmode:workspace-release-impact`        | Decide which crates need version bumps                       |
| `godmode:writing-solid-rust`              | SOLID principles and hexagonal arch in Rust                  |
| `godmode:depgraph`                        | Hexagonal architecture report for Rust workspace             |
| `godmode:open-knowledge-discovery`        | Install and use Open Knowledge on a repository               |
| `godmode:open-knowledge-write-skill`      | Author, draft, and install a new OpenKnowledge skill         |
| `godmode:agent-improvement-loop`          | Collect traces, feedback, evals, HALO diagnosis              |
| `godmode:1password-tailscale`             | SSH auth failures, credential lookup, tailnet access         |
| `godmode:async-sync-bridge`               | Mixing Tokio async with sync blocking I/O libraries          |
| `godmode:baml-iteration`                  | Edit/validate/test loop for devloop \*.baml files            |
| `godmode:chunked-file-reading`            | Reading large files that exceed context limits               |
| `godmode:crs-hook-testing`                | Adding/debugging a crs hook pipeline rule                    |
| `godmode:daily-orchestration`             | Daily maintenance across all repos                           |
| `godmode:devloop-analyze`                 | Running `devloop git analyze` on a repo                      |
| `godmode:devloop-bench-cycle`             | Full benchmark cycle — criterion, budgets, regressions       |
| `godmode:devloop-daily-update`            | Update daily note / standup from devloop analysis            |
| `godmode:devloop-standup`                 | Summarize recent repo activity / timeline view               |
| `godmode:doc-review`                      | Reviewing documentation changes                              |
| `godmode:doc-sync`                        | Syncing docs against source drift                            |
| `godmode:doc-writer`                      | Writing new documentation from scratch                       |
| `godmode:doob-triage`                     | Prioritized todo triage for the current project              |
| `godmode:env-chain-tracer`                | Tracing source_up chain for missing/wrong env vars           |
| `godmode:env-debug`                       | op run/direnv secret resolution failures                     |
| `godmode:herald-sync`                     | Cross-project activity synthesis at session end              |
| `godmode:maestro-dev-setup`               | Onboarding a new Maestro dev workstation                     |
| `godmode:minibox-ci`                      | minibox CI, self-hosted runner, xtask gate failures          |
| `godmode:minibox-dev`                     | minibox quality gates, crates/adapters, VPS testing          |
| `godmode:mise-toolchains`                 | Toolchain version conflicts, mise shim errors                |
| `godmode:obfsck-workflow`                 | obfsck feature work — ObfuscationLevel, PII gating           |
| `godmode:obsidian-vault`                  | Working in the Obsidian Vault directory                      |
| `godmode:pieces`                          | Working with Pieces on-device AI memory platform             |
| `godmode:pieces-health`                   | Pieces MCP timeout/disconnect — check/restart PiecesOS       |
| `godmode:pieces-ltm`                      | Historical context from Pieces long-term memory              |
| `godmode:rust-release-orchestrator`       | Coordinating a Rust workspace release                        |
| `godmode:rust-script`                     | Writing a standalone rust-script one-off utility             |
| `godmode:rust-snapshot-review`            | Reviewing insta .snap.new files after nextest                |
| `godmode:rust-unsafe-env-mutation`        | set_var/remove_var unsafe fn errors, env races in tests      |
| `godmode:secrets-management`              | Managing encrypted secrets, op/sops/age, SSH keys            |
| `godmode:session-to-skill`                | Extracting a repeated tool pattern into a new skill          |
| `godmode:think-consistency`               | Self-consistency reasoning across multiple paths             |
| `godmode:tool-presets`                    | Standardized tool set definitions for agents                 |
| `godmode:transparent-reader`              | Computing a side effect on streaming bytes without buffering |
| `godmode:using-conductor`                 | Run devloop → doob → devkit pipeline after a commit          |
| `godmode:using-devloop`                   | Development context via devloop's commit/session view        |
| `godmode:using-doob`                      | Managing todos/handoffs via doob CLI or doobdash             |
| `godmode:using-forge`                     | Primary dev companion for minibox/devloop/doob/devkit        |
| `godmode:using-gkg`                       | Structured knowledge graph of a codebase                     |
| `godmode:using-maestro`                   | Maestro project — K8s, Tilt, GKE, Go+Rust codegen            |
| `godmode:using-navigator`                 | Mental model briefing when jumping into a repo cold          |
| `godmode:using-sentinel`                  | Structured code review before a PR                           |
| `godmode:using-toolz`                     | System maintenance, log/db queries via toolz CLI             |
| `godmode:uv-script`                       | Writing a standalone Python script with uv/PEP 723           |
| `godmode:version-sync`                    | Codegen version mismatch (baml/build.rs/protoc)              |

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

- **`helpers/session-start.sh`** — run at session start to verify CLI and triage

## CLI Quick Reference

```bash
godmode handon                          # session start triage
godmode handoff                         # session end closeout
godmode plan ingest <plan.md>           # ingest plan → task graph
godmode task list [--json]              # all tasks
godmode task next [--json]              # next runnable (exit 1 if none)
godmode task add <title> [--id <id>] [opts]  # add task
godmode task start <id>                 # mark running
godmode task done <id> [--commit <sha>] # mark done
godmode task block <id> "<reason>"      # mark blocked
godmode task unblock <id>               # reset to pending
godmode task unblock-all                # reset ALL blocked tasks to pending
godmode task run <id>                   # run task's run: command
godmode task remove <id>               # remove a task from the graph
godmode dispatch [--max 5] [--json]     # parallel chains for orca-strait
godmode agent dispatch <path> [--max N] # ingest + dispatch in one step
godmode status                          # graph counts + next runnable
godmode context [--json]                # session context for hooks/subagents
godmode verify [--crate-name <crate>]   # nextest + clippy + fmt gate
```
