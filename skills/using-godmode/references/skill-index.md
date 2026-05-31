# Skill Index

## When to Invoke Each Skill

| Skill                                     | Trigger                                                           |
| ----------------------------------------- | ----------------------------------------------------------------- |
| `godmode:writing-plans`                   | Design approved, ready to implement multi-step task               |
| `godmode:cap`                             | "cap", "commit and push", "ship it"                               |
| `godmode:ci-fix`                          | CI failing, "fix CI", broken pipeline after push                  |
| `godmode:brainstorm`                      | "let's build", "add feature", "design X", new code                |
| `godmode:task-driven-development`         | TDD with serialized YAML task lists and sequential issue chains   |
| `godmode:systematic-debugging`            | Bug, test failure, panic, unexpected behaviour                    |
| `godmode:parallel-agents`                 | 2+ independent task chains or crates to implement                 |
| `godmode:verification-before-completion`  | Before any "done" claim, commit, or PR                            |
| `godmode:task-management`                 | Creating, tracking, or executing a task graph                     |
| `godmode:code-review`                     | Before merge, after feature complete                              |
| `godmode:introspection`                   | After adding skills, before plugin release, "audit godmode"       |
| `godmode:refactoring`                     | Restructuring code without changing behaviour                     |
| `godmode:receiving-review`                | Processing incoming review comments                               |
| `godmode:tackle-issues`                   | Working on GitHub issues in parallel                              |
| `godmode:observability-as-infrastructure` | Adding tracing to helpers and subagents                           |
| `godmode:testing-philosophy`              | Designing test strategy, reviewing test coverage                  |
| `godmode:wave-integration`                | Merging parallel agent branches into a single integration commit  |
| `godmode:moa`                             | Mixture-of-agents patterns for multi-model reasoning              |
| `godmode:todo-issue-sync`                 | Auditing inline TODOs and syncing uncovered items to issues       |
| `godmode:self-reflect`                    | End-of-session retrospective — "reflect", "what did we do"        |
| `godmode:merge`                           | Merge branch, create/merge PR, squash, worktree cleanup           |
| `godmode:rust-conventions`                | Rust conventions for writing/reviewing \*.rs code                 |
| `godmode:context-map`                     | Map relevant files before any implementation task                 |
| `godmode:decompose`                       | Split large diff/PR/branch into smaller independent units         |
| `godmode:doublecheck`                     | Three-layer verification pipeline for factual claims              |
| `godmode:mini-context-graph`              | Persistent knowledge base with entity graph and wiki pages        |
| `godmode:agent-governance`                | Governance, safety, and trust patterns for AI agent systems       |
| `godmode:memory-banking`                  | Generate/maintain .ctx/memory-banking/ with source-backed context |

## Skill Priority Order (by phase)

1. **ORIENT**: `task-management` (handon, triage), `memory-banking` (context injection)
2. **PLAN**: `brainstorm`, `writing-plans`, `context-map`
3. **ACT**: `task-driven-development`, `parallel-agents`, `systematic-debugging`
4. **VERIFY**: `verification-before-completion`, `code-review`, `testing-philosophy`
5. **SHIP**: `cap`, `merge`, `wave-integration`

## Red Flags (you are rationalising — stop)

| Thought                     | Truth                                |
| --------------------------- | ------------------------------------ |
| "This is simple"            | Check anyway                         |
| "I need context first"      | Skill check comes before exploration |
| "The skill is overkill"     | If it exists, use it                 |
| "Just this one thing first" | Check before acting                  |

## Skill Chain: Feature Development (by phase)

```
ORIENT  godmode handon → task-management (triage)
  ↓
PLAN    godmode:brainstorm → writing-plans → plan ingest
  ↓
ACT     task-management (start/done loop)
          → task-driven-development (per task)
            → parallel-agents (if independent chains)
  ↓
VERIFY  verification-before-completion → code-review
  ↓
SHIP    cap → merge → handoff
```
