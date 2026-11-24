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

### SOLID in Rust

**Single Responsibility Principle (SRP)**

- Each struct/enum has one reason to change
- Domain types contain business logic
- Adapters only handle external system integration
- No leakage between layers

**Open/Closed Principle (OCP)**

- Extend behavior via new trait implementations
- Add new adapters without modifying domain code
- Use generics and trait bounds for extensibility

**Liskov Substitution Principle (LSP)**

- Any implementor of a trait can replace another
- Trait contracts must be honored
- Type safety ensures substitutability

**Interface Segregation Principle (ISP)**

- Small, focused trait definitions
- Clients depend only on methods they use
- Compose larger behaviors from smaller traits

**Dependency Inversion Principle (DIP)**

- Domain depends on traits (abstractions), not concrete types
- Infrastructure implements traits
- Composition root (main.rs) wires dependencies

### Hexagonal Architecture

```text
┌─────────────────────────────────────────────────┐
│              Composition Root                   │
│                  (main.rs)                      │
│   Wires domain to adapters, injects deps        │
└────────────────┬────────────────────────────────┘
                 │
    ┌────────────┴────────────┐
    │                         │
┌───▼─────────────┐    ┌──────▼──────────────┐
│   Domain Layer  │    │ Infrastructure      │
│                 │    │    Adapters         │
│  ┌──────────┐   │    │                     │
│  │Business  │   │    │  ┌──────────────┐   │
│  │Logic     │───┼────┼─►│ OpenAI       │   │
│  │          │   │    │  │ Adapter      │   │
│  └──────────┘   │    │  └──────────────┘   │
│                 │    │                     │
│  ┌──────────┐   │    │  ┌──────────────┐   │
│  │Traits    │◄──┼────┼──│ Stripe       │   │
│  │(Ports)   │   │    │  │ Adapter      │   │
│  └──────────┘   │    │  └──────────────┘   │
│                 │    │                     │
│  ┌──────────┐   │    │  ┌──────────────┐   │
│  │Domain    │   │    │  │ Terminal     │   │
│  │Types     │   │    │  │ Renderer     │   │
│  └──────────┘   │    │  └──────────────┘   │
└─────────────────┘    └─────────────────────┘

Dependencies point inward: adapters → domain
Domain has zero dependencies on infrastructure
</text>
```

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

This skill includes three complete domain examples in the `references/` directory:

### 1. LLM Inference (`references/solid-LLM-inference.md`)

Demonstrates SOLID principles for AI/LLM integration:

- **Domain**: `LlmProvider` trait as port
- **Components**: `CodeReviewer` and `QaAgent` generic over providers
- **Adapters**: OpenAI, Claude (Anthropic), and Ollama implementations
- **Pattern**: Easily swap LLM providers without changing business logic

### 2. UI Components (`references/solid-UI-components.md`)

Shows render-agnostic UI component architecture:

- **Domain**: `Component` and `Renderer` traits
- **Components**: `Label`, `Button`, `LoginForm`
- **Adapters**: Terminal and Web renderers
- **Pattern**: Same UI components render to multiple backends

### 3. Payment Processing (`references/solid-payment-processing.md`)

Minimal payment processing example:

- **Domain**: `PaymentProcessor` trait
- **Components**: `CheckoutService` with business rules
- **Adapters**: Stripe and InMemory implementations
- **Pattern**: Swap payment processors, use test doubles easily

## Code Examples

The code for each step, pattern, test, and mistake lives in `references/`. Read the file in the right column before writing that kind of code. Each file opens with the same "Read this before" line.

| File                                            | Read before                                                                           |
| ----------------------------------------------- | ------------------------------------------------------------------------------------- |
| `references/walkthrough-domain.md`              | writing the domain port, domain types, or domain service (Steps 1 to 3)               |
| `references/walkthrough-adapters.md`            | writing infrastructure adapters or wiring them in `main.rs` (Steps 4 and 5)           |
| `references/pattern-async-traits.md`            | a port trait has async methods                                                        |
| `references/pattern-builder.md`                 | an adapter needs several configuration values, some required                          |
| `references/pattern-multiple-traits.md`         | one trait has grown methods that callers do not all use (ISP)                         |
| `references/pattern-error-mapping.md`           | an adapter must map infrastructure errors to domain errors                            |
| `references/testing-unit.md`                    | writing unit tests for domain services with test doubles                              |
| `references/testing-integration.md`             | testing an adapter against a real system or test instance                             |
| `references/refactoring-walkthrough.md`         | converting a tightly coupled service to ports and adapters (Refactoring Steps 2 to 4) |
| `references/mistake-trait-mirrors-api.md`       | designing a port from an external API's method list                                   |
| `references/mistake-domain-uses-infra-types.md` | a domain signature mentions an SDK type                                               |
| `references/mistake-generic-explosion.md`       | a service struct has four or more generic parameters                                  |
| `references/mistake-logic-in-adapter.md`        | an adapter method contains a validation or business rule                              |

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

1. **Choose concrete implementations.** Code: `references/walkthrough-adapters.md`.

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

### Domain Layer

- **Zero external dependencies** - domain should compile standalone
- **Rich domain types** - use newtype pattern for domain concepts
- **Explicit errors** - define domain-specific error types
- **Pure functions where possible** - easier to test and reason about

### Traits (Ports)

- **Small and focused** - follow Interface Segregation Principle
- **Avoid leaky abstractions** - no infrastructure details in trait methods
- **Use associated types** when return types vary by implementation
- **Document trait contracts** - preconditions, postconditions, invariants

### Adapters

- **One adapter per external system** - SRP for infrastructure
- **Map errors to domain errors** - don't leak infrastructure error types
- **Configuration via constructor** - make dependencies explicit
- **Implement multiple traits** - adapter can implement several ports

### Composition Root

- **All wiring happens here** - main.rs or application setup
- **Read configuration** - environment variables, config files
- **Create all instances** - adapters, services, domain objects
- **No business logic** - only dependency injection

### Testing

- **Mock at trait boundaries** - easy to create test doubles
- **Unit test domain logic** - fast, no external dependencies
- **Integration test adapters** - slower, against real systems
- **Property-based testing** - use proptest for domain invariants

## Refactoring Existing Code

### Step 1: Identify External Dependencies

Find all direct usage of:

- HTTP clients
- Database connections
- File system operations
- Third-party APIs

### Step 2: Extract Traits

For each dependency, create a trait. Code: `references/refactoring-walkthrough.md`.

### Step 3: Implement Adapters

Create adapter structs that implement the traits. Code: `references/refactoring-walkthrough.md`.

### Step 4: Update Composition Root

Wire the new structure in main.rs. Code: `references/refactoring-walkthrough.md`.

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

SOLID principles in Rust:

- **Use traits as ports** between domain and infrastructure
- **Keep domain pure** with zero external dependencies
- **Implement adapters** for each external system
- **Wire dependencies** in composition root (main.rs)
- **Test easily** by mocking at trait boundaries
- **Extend without modification** by adding new trait implementations

This architecture maximizes:

- Testability (easy mocking)
- Maintainability (clear separation of concerns)
- Flexibility (swap implementations)
- Team scalability (clear boundaries)

Remember: **Dependencies point inward.** Domain depends on nothing. Infrastructure depends on domain traits.

## Additional Resources

For complete working examples, see:

- `references/solid-LLM-inference.md` - AI/LLM port and adapters
- `references/solid-UI-components.md` - Render-agnostic UI components
- `references/solid-payment-processing.md` - Payment processing example

Each reference includes full code examples with explanations of where SOLID principles appear.
