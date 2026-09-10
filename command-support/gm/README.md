# command-support/gm/

Slash commands for Claude Code. Each `.yaml` file maps to a `/gm:<name>` command in the
command palette. The `.md` files in `commands/` are generated from these sources —
do not edit them directly.

## Atomic Commands

Single-skill wrappers for direct invocation.

| Command               | Skill                                                                     | Purpose                                       |
| --------------------- | ------------------------------------------------------------------------- | --------------------------------------------- |
| `/gm:cap`             | `godmode:cap`                                                             | Commit and push with cargo gates              |
| `/gm:tdd`             | `godmode:task-driven-development`                                         | Strict TDD — failing test before code         |
| `/gm:debug`           | `godmode:systematic-debugging`                                            | Root cause before fix                         |
| `/gm:trace`           | `godmode:observability-as-infrastructure`                                 | Query session trace log                       |
| `/gm:refactor`        | `godmode:refactoring`, `godmode:code-review`                              | Restructure without changing behaviour        |
| `/gm:review-code`     | `godmode:code-review`, `godmode:verification-before-completion`           | Quality pass before merge                     |
| `/gm:doc-review`      | `godmode:doc-review`, `godmode:doc-writer`, `godmode:cap`                 | Review docs, then commit and push             |
| `/gm:doc-sync`        | `godmode:doc-review`, `godmode:doc-sync`, `godmode:doc-writer`            | Detect and fix doc/code drift                 |
| `/gm:doc-writer`      | `godmode:doc-review`, `godmode:doc-writer`                                | Write new documentation from scratch          |
| `/gm:ci-fix`          | `godmode:ci-fix`                                                          | Fix a failing CI pipeline                     |
| `/gm:self-heal`       | `godmode:ci-fix` (loop)                                                   | Self-healing CI loop until all gates pass     |
| `/gm:brainstorm`      | `godmode:brainstorm`                                                      | Explore approaches and obtain approval        |
| `/gm:design`          | `godmode:design`                                                          | Write an approved architectural design        |
| `/gm:plan`            | `godmode:writing-plans`                                                   | Scaffold an implementation plan               |
| `/gm:ingest`          | `godmode:ingest`                                                          | Load a completed plan into the task graph     |
| `/gm:implement`       | `task-management` → `task-driven-development`                             | Execute the ingested task graph with TDD      |
| `/gm:test-fix-commit` | —                                                                         | Test → fix → commit cycle                     |
| `/gm:tackle-issues`   | `godmode:tackle-issues`                                                   | Work GitHub issues in parallel worktrees      |
| `/gm:moa-review`      | `godmode:gm-crate` (dispatched directly, no `moa` skill call)             | Multi-lens review + parallel fix agents       |
| `/gm:preflight`       | —                                                                         | Pre-session environment checks                |
| `/gm:handon`          | —                                                                         | Session start — triage outstanding work       |
| `/gm:handoff`         | —                                                                         | Session end — write HANDOFF state             |
| `/gm:fresh-branch`    | —                                                                         | Create a clean branch from latest main        |
| `/gm:introspect`      | `godmode:introspection`                                                   | Audit plugin files for conformance            |
| `/gm:dispatch-all`    | `godmode:gm-crate` (dispatched directly, no `parallel-agents` skill call) | Fan out open GitHub issues to parallel agents |
| `/gm:auth-fail-fast`  | —                                                                         | Detect and surface auth failures early        |

## Workflow Commands

Multi-skill pipelines that chain skills in sequence.

| Command               | Pipeline                                                                                                                 |
| --------------------- | ------------------------------------------------------------------------------------------------------------------------ |
| `/gm:ideate`          | repo scan → gap analysis → `brainstorm`                                                                                  |
| `/gm:feature`         | `brainstorm` → `design` → `writing-plans` → `ingest` → `tdd` → `verify` → `cap`                                          |
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

## Legacy path compatibility

`command-support/gm/*.yaml` is the canonical source. The tracked `commands/gm/*.yaml` mirrors keep
older readers working, and `commands/gm/generate.nu` forwards the former generator path to the
canonical wrapper. Generation refreshes the mirrors from canonical YAML before rendering. New
tooling must use `command-support/gm/`; mirrored files must not become a second source of truth.

## Adding a Command

1. Create `command-support/gm/<name>.yaml` with fields: `name`, `description`, `template`,
   `prompt`, `allowedTools`, and `maxTurns`. `description` is canonical, required repository
   metadata; do not rely on the renderer’s compatibility fallback to the first prompt line.
2. Use `template: dev` for implementation commands, `template: debug` for diagnostic ones.
3. Reference skills as `godmode:<skill-name>` in the prompt body.
4. Run `nu command-support/gm/generate.nu`; do not create the `.md` file manually. The wrapper
   delegates to the Rust renderer.

Generate or install commands directly with:

```bash
godmode command generate --target claude
godmode command generate --target opencode --output-dir /tmp/godmode-opencode-commands
godmode command install-opencode
godmode command install-opencode --dry-run
```
