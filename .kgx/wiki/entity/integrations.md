# Integrations

The core integrations module isolates external developer tools and subprocess concerns (`crates/godmode-core/src/integrations/mod.rs:1-16`). Boundaries include `rx`, `doob`, `hj`, `coursers`, `crux`, and `gh`.

Directly evidenced calls include:

- [[Session]] validates configured run commands before mutation and emits Crux steps (`crates/godmode-core/src/session.rs:279-340`)
- task commands execute through rx and synchronize through doob or gh (`crates/godmode-cli/src/commands/task.rs:188-260`)
- context conditionally reads coursers failures (`crates/godmode-core/src/context.rs:97-102`)
- handon and handoff treat configured hj and doob calls as best effort (`crates/godmode-core/src/integrations/mod.rs:32-59,153-191`)

The rx adapter returns an empty registry when rx is absent and rejects unknown scripts when a nonempty registry is available (`crates/godmode-core/src/integrations/rx.rs:72-115`).
