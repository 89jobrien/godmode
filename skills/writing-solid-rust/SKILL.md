---
name: writing-solid-rust
description: Apply SOLID principles and hexagonal architecture to Rust code. Teaches trait-based ports, domain-driven design, dependency inversion, and clean separation between business logic and infrastructure. Use when designing Rust systems, refactoring for testability, or implementing pluggable architectures.
allowed-tools: [Read, Write, Edit, Bash, Grep, Glob]
---

# Writing SOLID Rust

You are an expert in SOLID principles and hexagonal architecture applied to Rust, specializing in trait-based
ports and adapters, dependency inversion, and clean separation of concerns.

## When to Use This Skill

Use this skill whenever:

- Designing new Rust systems or modules
- Refactoring existing Rust code for better testability and maintainability
- Implementing pluggable architectures (multiple backends, swappable components)
- Working on domain-driven design in Rust
- The user mentions SOLID, hexagonal architecture, ports and adapters, or clean architecture
- You need to separate business logic from infrastructure concerns

## Core Principles

The SOLID principles as they apply to Rust are in `references/principles-solid-in-rust.md`. Read it before explaining a principle or applying one to a design.

The hexagonal layout and its dependency rule are in `references/principles-hexagonal-architecture.md`. Read it before drawing a layout for a new system.

Dependencies point inward: adapters depend on domain traits, and the domain depends on no infrastructure.

## Architecture Pattern

### Layer Structure

**1. Domain Layer** (`domain.rs`)

- Pure business logic
- No external dependencies
- Defines traits (ports)
- Domain types and errors
- Core business rules

**2. Domain Components** (e.g., `agents.rs`, `components.rs`)

- Business logic that uses domain traits
- Generic over trait implementations
- No knowledge of specific adapters

**3. Infrastructure Adapters** (`infra/` directory)

- Implement domain traits
- Handle external system integration
- One adapter per external dependency
- Contains framework/library code

**4. Composition Root** (`main.rs`)

- Wire domain to concrete adapters
- Inject dependencies
- Application entry point
- Only place that knows about all layers

## Reference Examples

Three complete domain examples (LLM inference, UI components, payment processing) are in `references/reference-examples-overview.md`. Read it before choosing one to copy from.

## Code Examples

The code for each step, pattern, test, and mistake lives in `references/`. Read the file in the right column before writing that kind of code. Each file opens with the same "Read this before" line.

| File                                                       | Read before                                                                                                           |
| ---------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------- |
| `references/walkthrough-domain.md`                         | writing the domain port, domain types, or domain service (Steps 1 to 3)                                               |
| `references/walkthrough-adapters.md`                       | writing infrastructure adapters for the payment example (Step 4)                                                      |
| `references/walkthrough-composition-root.md`               | wiring adapters into a composition root for the payment example (Step 5)                                              |
| `references/pattern-async-traits.md`                       | a port trait has async methods                                                                                        |
| `references/pattern-builder.md`                            | an adapter needs several configuration values, some required                                                          |
| `references/pattern-multiple-traits.md`                    | one trait has grown methods that callers do not all use (ISP)                                                         |
| `references/pattern-error-mapping.md`                      | an adapter must map infrastructure errors to domain errors                                                            |
| `references/testing-unit.md`                               | writing unit tests for domain services with test doubles                                                              |
| `references/testing-integration.md`                        | testing an adapter against a real system or test instance                                                             |
| `references/refactoring-walkthrough.md`                    | converting a tightly coupled service to ports and adapters (Refactoring Step 2)                                       |
| `references/refactoring-walkthrough-implement-and-wire.md` | implementing the adapters or rewiring the composition root in a trait-extraction refactor (Refactoring Steps 3 and 4) |
| `references/mistake-trait-mirrors-api.md`                  | designing a port from an external API's method list                                                                   |
| `references/mistake-domain-uses-infra-types.md`            | a domain signature mentions an SDK type                                                                               |
| `references/mistake-generic-explosion.md`                  | a service struct has four or more generic parameters                                                                  |
| `references/mistake-logic-in-adapter.md`                   | an adapter method contains a validation or business rule                                                              |
| `references/principles-solid-in-rust.md`                   | explaining a SOLID principle, or applying one to a Rust design                                                        |
| `references/principles-hexagonal-architecture.md`          | drawing the hexagonal layout for a new system, or deciding which layer a type or trait belongs to                     |
| `references/reference-examples-overview.md`                | choosing one of the three full domain examples to copy from                                                           |
| `references/best-practices.md`                             | reviewing a domain layer, port trait, adapter, composition root, or test setup against the rules                      |
| `references/refactoring-existing-code-steps.md`            | reading the four steps for extracting traits from an existing service (Refactoring Steps 1 to 4)                      |
| `references/summary-and-benefits.md`                       | writing a closing summary of the architecture, or justifying it to a team                                             |

## Implementation Workflow

### Step 1: Identify the Port

1. **What external system do you need to abstract?**
   - Database access
   - HTTP API calls
   - File system operations
   - Message queues
   - Third-party services

2. **Define the domain trait (port)** in `domain.rs`. Code: `references/walkthrough-domain.md`.

### Step 2: Define Domain Types

1. **Create domain-specific types.** Code: `references/walkthrough-domain.md`.

2. **Keep types focused and minimal**
   - Only fields relevant to the domain
   - No infrastructure details

### Step 3: Implement Domain Logic

1. **Write business logic generic over the port.** Code: `references/walkthrough-domain.md`.

### Step 4: Create Infrastructure Adapters

1. **Implement the trait for each external system.** Code: `references/walkthrough-adapters.md`.

2. **Create test doubles for testing.** Code: `references/walkthrough-adapters.md`.

### Step 5: Wire Dependencies in Composition Root

1. **Choose concrete implementations.** Code: `references/walkthrough-composition-root.md`.

## Common Patterns

### Pattern 1: Async Traits

When working with async code. Code: `references/pattern-async-traits.md`.

### Pattern 2: Builder for Complex Adapters

When adapters need configuration. Code: `references/pattern-builder.md`.

### Pattern 3: Multiple Traits for Different Capabilities

When components need different capabilities. Code: `references/pattern-multiple-traits.md`.

### Pattern 4: Error Type Mapping

Keep domain errors separate from infrastructure errors. Code: `references/pattern-error-mapping.md`.

## Testing Strategy

### Unit Testing Domain Logic

Domain logic is easy to test with test doubles. Code: `references/testing-unit.md`.

### Integration Testing Adapters

Test adapters against real systems (or test instances). Code: `references/testing-integration.md`.

## Best Practices

The rules for the domain layer, traits, adapters, composition root, and testing are in `references/best-practices.md`. Read it before reviewing code against them.

## Refactoring Existing Code

The four steps for extracting traits from an existing service are in `references/refactoring-existing-code-steps.md`. Read it first. The code for each step is in the refactoring rows of the index above.

## Common Mistakes to Avoid

### Mistake 1: Traits That Mirror Infrastructure APIs

Code: `references/mistake-trait-mirrors-api.md`.

### Mistake 2: Domain Depending on Infrastructure Types

Code: `references/mistake-domain-uses-infra-types.md`.

### Mistake 3: Too Many Generic Parameters

Code: `references/mistake-generic-explosion.md`.

### Mistake 4: Putting Business Logic in Adapters

Code: `references/mistake-logic-in-adapter.md`.

## When NOT to Use This Pattern

This architecture has overhead. Avoid it when:

- **Prototyping or spike work** - get feedback first, refactor later
- **Single implementation** - if you'll only ever have one implementation, trait might be premature
- **Simple scripts** - for one-off utilities, direct dependencies are fine
- **Performance-critical hot paths** - dynamic dispatch has cost (though often negligible)

In these cases, prefer simpler approaches and refactor to SOLID/hexagonal when:

- You need to swap implementations
- Testing becomes difficult
- The codebase grows in complexity
- Multiple teams work on different layers

## Summary

The SOLID principles in Rust, and the benefits of this architecture, are in `references/summary-and-benefits.md`.

Remember: **Dependencies point inward.** Domain depends on nothing. Infrastructure depends on domain traits.
