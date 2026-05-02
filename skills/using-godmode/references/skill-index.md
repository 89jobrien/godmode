# Skill Index

## When to Invoke Each Skill

| Skill                                     | Trigger                                                     |
| ----------------------------------------- | ----------------------------------------------------------- |
| `godmode:cap`                             | "cap", "commit and push", "ship it"                         |
| `godmode:ci-fix`                          | CI failing, "fix CI", broken pipeline after push            |
| `godmode:brainstorm`                      | "let's build", "add feature", "design X", new code          |
| `godmode:writing-plans`                   | Design approved, ready to implement multi-step task         |
| `godmode:test-driven-development`         | Any task that produces production Rust code                 |
| `godmode:systematic-debugging`            | Bug, test failure, panic, unexpected behaviour              |
| `godmode:parallel-agents`                 | 2+ independent task chains or crates to implement           |
| `godmode:verification-before-completion`  | Before any "done" claim, commit, or PR                      |
| `godmode:task-management`                 | Creating, tracking, or executing a task graph               |
| `godmode:code-review`                     | Before merge, after feature complete                        |
| `godmode:introspection`                   | After adding skills, before plugin release, "audit godmode" |
| `godmode:refactoring`                     | Restructuring code without changing behaviour               |
| `godmode:receiving-review`                | Processing incoming review comments                         |
| `godmode:tackle-issues`                   | Working on GitHub issues in parallel                        |
| `godmode:observability-as-infrastructure` | Adding tracing to helpers and subagents                     |
| `godmode:testing-philosophy`              | Designing test strategy, reviewing test coverage            |

## Skill Priority Order

1. Process skills first: `brainstorm`, `systematic-debugging`
2. Implementation skills second: `test-driven-development`, `parallel-agents`
3. Quality gates last: `verification-before-completion`, `code-review`

## Red Flags (you are rationalising — stop)

| Thought                     | Truth                                |
| --------------------------- | ------------------------------------ |
| "This is simple"            | Check anyway                         |
| "I need context first"      | Skill check comes before exploration |
| "The skill is overkill"     | If it exists, use it                 |
| "Just this one thing first" | Check before acting                  |

## Skill Chain: Feature Development

```
godmode:brainstorm
  → writing-plans
    → plan ingest (godmode CLI)
      → task-management (handon, next, start, done loop)
        → test-driven-development (per task)
          → parallel-agents (if independent chains exist)
            → verification-before-completion
              → code-review
                → handoff (godmode CLI)
```
