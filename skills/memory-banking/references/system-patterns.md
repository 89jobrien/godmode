# System Patterns

## Architecture

<!-- high-level layout: crates, modules, services, boundaries -->
<!-- For Rust projects, identify: domain layer, adapter layer, composition root -->

## Data flow

<!-- how data moves through the system; key entry/exit points -->
<!-- For hexagonal: which ports (traits) mediate inbound vs outbound flow? -->

## Patterns to follow

- <!-- naming conventions -->
- <!-- error handling style (domain errors vs adapter errors, mapping strategy) -->
- <!-- testing approach (mock at trait boundaries, proptest for invariants) -->
- <!-- module organization (domain/, infra/, ports as traits in domain) -->
- <!-- dependency direction: adapters depend on domain traits, never the reverse -->

### SOLID conventions (Rust projects)

- **SRP**: each struct/module has one reason to change
- **OCP**: extend via new trait impls, not modifying existing code
- **LSP**: all trait implementors are substitutable (honour contracts)
- **ISP**: small focused traits; clients depend only on methods they use
- **DIP**: domain defines traits (ports); infra implements them; main.rs wires

### Hexagonal structure (if applicable)

```
domain/        — pure business logic, traits (ports), domain types, zero deps
infra/         — adapters per external system (DB, API, FS, CLI)
main.rs        — composition root; creates adapters, injects into domain
```

- Traits live in domain, implementations live in infra
- Domain errors are distinct from infrastructure errors; adapters map between them
- Test doubles implement domain traits directly (no mock framework needed)

## Patterns to avoid

- <!-- anti-patterns specific to this codebase -->
- Traits that mirror external API shapes (leaky abstractions)
- Business logic inside adapters
- Domain types that reference infrastructure types (e.g. `StripeToken` in domain)
- Generic parameter explosion — use trait objects or a `Dependencies` struct when >3

## Key files

| Path                   | Purpose                              |
| ---------------------- | ------------------------------------ |
| <!-- src/lib.rs -->    | <!-- entry point -->                 |
| <!-- src/domain.rs --> | <!-- traits/ports + domain types --> |
| <!-- src/infra/ -->    | <!-- adapter implementations -->     |

_Source: source code structure, CLAUDE.md conventions. Update on architecture decisions._
