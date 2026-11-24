# Best practices

Read this before reviewing a domain layer, port trait, adapter, composition root, or test setup against the rules.

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
