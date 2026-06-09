# Design Principles

## Hexagonal Architecture

External dependencies (storage, processes, network, clock) go behind a trait (port). Business
logic is generic over the port trait. Concrete implementations (adapters) live in a separate
module or crate from the port definition.

```
         ┌──────────────────────────────┐
         │         Domain Logic          │
         │   (generic over port traits) │
         └──────────┬───────────────────┘
                    │  uses
          ┌─────────▼─────────┐
          │    Port (trait)    │  ← defined in the owning crate
          └─────────┬─────────┘
                    │  implemented by
          ┌─────────▼─────────┐
          │  Adapter (impl)    │  ← defined in infra/integration crate
          └───────────────────┘
```

**Rule**: if you're calling `std::fs`, `std::process::Command`, or any network crate directly
from domain logic, you need a port.

## YAGNI

Design only what the current requirement needs. No "we might need this later" fields, methods,
or trait bounds. Every element in the design doc must map to at least one task in the plan.

## Naming Conventions (godmode workspace)

| Kind            | Convention                      | Example                                |
| --------------- | ------------------------------- | -------------------------------------- |
| Traits          | `PascalCase`, noun or adjective | `TaskRunner`, `Traceable`              |
| Structs         | `PascalCase`, noun              | `TaskGraph`, `SessionSummary`          |
| Enums           | `PascalCase`, noun              | `Status`, `HookEvent`                  |
| Functions       | `snake_case`, verb              | `run_task`, `load_graph`               |
| Modules         | `snake_case`, noun              | `graph`, `session_trace`               |
| CLI subcommands | `kebab-case`                    | `task-management`, `plan-ingest`       |
| Skill names     | `kebab-case`                    | `godmode:design`, `godmode:brainstorm` |

## API Surface Rules

- Make types and functions `pub` only if a downstream crate needs them.
- Prefer `pub(crate)` for internal helpers.
- New traits should have the minimum method set required — methods can be added, not removed.
- Avoid `pub` fields on structs with invariants; use accessor methods.

## Semver Implications

Any change to a `pub` type, trait, or function signature in `godmode-core` is a breaking change
for downstream consumers. Before adding to the public API, ask: does this need to be public, or
can it be `pub(crate)`?
