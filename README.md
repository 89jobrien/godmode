# godmode

Self-contained Rust-native development methodology plugin for Claude Code. Replaces
superpowers entirely. No external tool dependencies.

## Skills

| Skill                            | Purpose                                                    |
| -------------------------------- | ---------------------------------------------------------- |
| `using-godmode`                  | Session orientation, available skills, always-active rules |
| `test-driven-development`        | RED-GREEN-REFACTOR with cargo/nextest/clippy               |
| `systematic-debugging`           | 4-phase root cause analysis                                |
| `brainstorming`                  | Design-first gate before any implementation                |
| `writing-plans`                  | Complete implementation plans with exact code and paths    |
| `verification-before-completion` | Evidence before claims                                     |
| `task-management`                | Causal task graph in `.ctx/GODMODE.tasks.yaml`             |
| `parallel-agents`                | Concurrent crate-level TDD agents                          |

## Agents

| Agent             | Purpose                                   |
| ----------------- | ----------------------------------------- |
| `tdd-crate-agent` | TDD implementation in a single Rust crate |

## Task File

`.ctx/GODMODE.tasks.yaml` — ephemeral task graph, gitignored. Persists across sessions.
Drives sequential and parallel execution via causal `depends_on` chains.

## Install

```bash
claude plugin install /Users/joe/dev/godmode
```

Or register as local plugin via bazaar/local-marketplace.

## License

MIT OR Apache-2.0
