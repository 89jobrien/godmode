# commands/gm/

Slash commands for Claude Code. Each `.yaml` file maps to a `/gm:<name>` command in the
command palette. The `.md` files in `commands/` are generated from these sources —
do not edit them directly.

## Atomic Commands

Single-skill wrappers for direct invocation.

| Command               | Skill                                     | Purpose                                      |
| --------------------- | ----------------------------------------- | -------------------------------------------- |
| `/gm:cap`             | `godmode:cap`                             | Commit and push with cargo gates             |
| `/gm:tdd`             | `godmode:task-driven-development`         | Strict TDD — failing test before code        |
| `/gm:debug`           | `godmode:systematic-debugging`            | Root cause before fix                        |
| `/gm:trace`           | `godmode:observability-as-infrastructure` | Query session trace log                      |
| `/gm:refactor`        | `godmode:refactoring`                     | Restructure without changing behaviour       |
| `/gm:review-code`     | `godmode:code-review`                     | Quality pass before merge                    |
| `/gm:ci-fix`          | `godmode:ci-fix`                          | Fix a failing CI pipeline                    |
| `/gm:self-heal`       | `godmode:ci-fix` (loop)                   | Self-healing CI loop until all gates pass    |
| `/gm:plan`            | `godmode:writing-plans`                   | Scaffold an implementation plan              |
| `/gm:test-fix-commit` | —                                         | Test → fix → commit cycle                    |
| `/gm:tackle-issues`   | `godmode:tackle-issues`                   | Work GitHub issues in parallel worktrees     |
| `/gm:moa-review`      | `godmode:moa`                             | Multi-model review via mixture of agents     |
| `/gm:preflight`       | —                                         | Pre-session environment checks               |
| `/gm:handon`          | —                                         | Session start — triage outstanding work      |
| `/gm:handoff`         | —                                         | Session end — write HANDOFF state            |
| `/gm:fresh-branch`    | —                                         | Create a clean branch from latest main       |
| `/gm:introspect`      | `godmode:introspection`                   | Audit plugin files for conformance           |
| `/gm:dispatch-all`    | `godmode:parallel-agents`                 | Fan out all pending tasks to parallel agents |
| `/gm:auth-fail-fast`  | —                                         | Detect and surface auth failures early       |

## Workflow Commands

Multi-skill pipelines that chain skills in sequence.

| Command               | Pipeline                                                                                                                 |
| --------------------- | ------------------------------------------------------------------------------------------------------------------------ |
| `/gm:brainstorm`      | `brainstorm` → `design`                                                                                                  |
| `/gm:ideate`          | repo scan → gap analysis → `brainstorm`                                                                                  |
| `/gm:feature`         | `brainstorm` → `design` → `writing-plans` → `tdd` → `verify` → `cap`                                                     |
| `/gm:ship`            | `verification-before-completion` → `changelog` → `release-notes` → `cap`                                                 |
| `/gm:release`         | `release-readiness-check` → `workspace-release-impact` → `workspace-bump-commit` → `changelog` → `release-notes` → `cap` |
| `/gm:audit`           | `health-score` → `dead-code` → `dep-audit` → `mistake-tracker` → `repo-gap-backlog`                                      |
| `/gm:pr`              | `code-review` → `doublecheck` → `pr-author` → `merge`                                                                    |
| `/gm:review-incoming` | `receiving-review` → `verification-before-completion` → `cap`                                                            |
| `/gm:deps`            | `dep-audit` → `dep-bump` → `cap`                                                                                         |
| `/gm:session-end`     | `whatidid` → `self-reflect` → `mistake-tracker` → `memory-banking` → `session-wrap-commit-push`                          |
| `/gm:improve-agent`   | `self-reflect` → `pattern-learner` → `agent-improvement-loop` → `agents-skill-save`                                      |
| `/gm:context`         | `context-map` → `memory-banking` → `mini-context-graph`                                                                  |
| `/gm:debug-loop`      | `systematic-debugging` → `doublecheck` → `verification-before-completion` → `cap`                                        |
| `/gm:doc-enrich`      | `doc-review` → `doc-maintainer` → `doc-sync` → `cap`                                                                     |
| `/gm:polish`          | `refactoring` → `testing-philosophy` → `rustqual` → `release-readiness-check` → `changelog` → `cap`                      |
| `/gm:issues`          | `issue-triage` → `tackle-issues` → `todo-issue-sync` → `cap`                                                             |
| `/gm:observe`         | `observability-as-infrastructure` → `health-score` → `issue-triage` (read-only)                                          |

## Adding a Command

1. Create `commands/gm/<name>.yaml` with fields: `name`, `template`, `prompt`,
   `allowedTools`, `maxTurns`.
2. Use `template: dev` for implementation commands, `template: debug` for diagnostic ones.
3. Reference skills as `godmode:<skill-name>` in the prompt body.
4. The `.md` file in `commands/` is auto-generated — do not create it manually.
