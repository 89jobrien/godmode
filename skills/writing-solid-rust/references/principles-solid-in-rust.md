# SOLID principles in Rust

Read this before explaining a SOLID principle, or before applying one to a Rust design.

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
