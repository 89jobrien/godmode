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

## Implementation Workflow

### Step 1: Identify the Port

1. **What external system do you need to abstract?**
   - Database access
   - HTTP API calls
   - File system operations
   - Message queues
   - Third-party services

2. **Define the domain trait (port)**

```rust
// domain.rs
pub trait PaymentProcessor {
    fn process(&self, payment: &Payment) -> Result<(), PaymentError>;
}
```

### Step 2: Define Domain Types

1. **Create domain-specific types**

```rust
// domain.rs
pub struct Payment {
    pub amount_cents: u64,
    pub currency: String,
}

#[derive(Debug)]
pub enum PaymentError {
    Rejected,
    Network,
}
```

2. **Keep types focused and minimal**
   - Only fields relevant to the domain
   - No infrastructure details

### Step 3: Implement Domain Logic

1. **Write business logic generic over the port**

```rust
// domain.rs or service.rs
pub struct CheckoutService<P: PaymentProcessor> {
    processor: P,
}

impl<P: PaymentProcessor> CheckoutService<P> {
    pub fn new(processor: P) -> Self {
        Self { processor }
    }

    pub fn checkout(&self, amount_cents: u64, currency: &str) -> Result<(), PaymentError> {
        // Business rules here
        let payment = Payment { amount_cents, currency: currency.to_string() };
        self.processor.process(&payment)
    }
}
```

### Step 4: Create Infrastructure Adapters

1. **Implement the trait for each external system**

```rust
// infra/stripe.rs
use crate::domain::{Payment, PaymentError, PaymentProcessor};

pub struct StripeProcessor {
    pub api_key: String,
}

impl PaymentProcessor for StripeProcessor {
    fn process(&self, payment: &Payment) -> Result<(), PaymentError> {
        // Call Stripe API
        println!("Stripe: charging {} {}", payment.amount_cents, payment.currency);
        Ok(())
    }
}
```

2. **Create test doubles for testing**

```rust
// infra/in_memory.rs
pub struct InMemoryProcessor {
    pub accepted: Vec<Payment>,
}

impl PaymentProcessor for InMemoryProcessor {
    fn process(&self, payment: &Payment) -> Result<(), PaymentError> {
        // Store in memory for testing
        Ok(())
    }
}
```

### Step 5: Wire Dependencies in Composition Root

1. **Choose concrete implementations**

```rust
// main.rs
mod domain;
mod infra {
    pub mod stripe;
    pub mod in_memory;
}

use infra::stripe::StripeProcessor;
use domain::CheckoutService;

fn main() {
    let processor = StripeProcessor {
        api_key: "sk_live_...".to_string(),
    };

    let checkout = CheckoutService::new(processor);
    checkout.checkout(5000, "USD").unwrap();
}
```

## Common Patterns

### Pattern 1: Async Traits

When working with async code:

```rust
use async_trait::async_trait;

#[async_trait]
pub trait LlmProvider {
    async fn chat(&self, messages: &[ChatMessage]) -> Result<String, LlmError>;
}

#[async_trait]
impl LlmProvider for OpenAiAdapter {
    async fn chat(&self, messages: &[ChatMessage]) -> Result<String, LlmError> {
        // Async implementation
    }
}
```

### Pattern 2: Builder for Complex Adapters

When adapters need configuration:

```rust
pub struct OpenAiAdapter {
    client: Client,
    api_key: String,
    model: String,
    timeout: Duration,
}

impl OpenAiAdapter {
    pub fn builder() -> OpenAiAdapterBuilder {
        OpenAiAdapterBuilder::default()
    }
}

pub struct OpenAiAdapterBuilder {
    api_key: Option<String>,
    model: String,
    timeout: Duration,
}

impl OpenAiAdapterBuilder {
    pub fn api_key(mut self, key: String) -> Self {
        self.api_key = Some(key);
        self
    }

    pub fn model(mut self, model: String) -> Self {
        self.model = model;
        self
    }

    pub fn build(self) -> Result<OpenAiAdapter, BuildError> {
        Ok(OpenAiAdapter {
            client: Client::new(),
            api_key: self.api_key.ok_or(BuildError::MissingApiKey)?,
            model: self.model,
            timeout: self.timeout,
        })
    }
}
```

### Pattern 3: Multiple Traits for Different Capabilities

When components need different capabilities:

```rust
// Instead of one large trait
pub trait Storage {
    fn read(&self, key: &str) -> Result<Vec<u8>, Error>;
    fn write(&self, key: &str, data: &[u8]) -> Result<(), Error>;
    fn delete(&self, key: &str) -> Result<(), Error>;
    fn list(&self) -> Result<Vec<String>, Error>;
}

// Use separate traits (ISP)
pub trait Reader {
    fn read(&self, key: &str) -> Result<Vec<u8>, Error>;
}

pub trait Writer {
    fn write(&self, key: &str, data: &[u8]) -> Result<(), Error>;
}

pub trait Deleter {
    fn delete(&self, key: &str) -> Result<(), Error>;
}

// Components depend only on what they need
pub struct ReadOnlyService<R: Reader> {
    reader: R,
}
```

### Pattern 4: Error Type Mapping

Keep domain errors separate from infrastructure errors:

```rust
// Domain error
#[derive(Debug)]
pub enum PaymentError {
    Rejected,
    Network,
    InvalidAmount,
}

// Adapter maps infrastructure errors to domain errors
impl PaymentProcessor for StripeProcessor {
    fn process(&self, payment: &Payment) -> Result<(), PaymentError> {
        self.stripe_client
            .charge(payment)
            .map_err(|e| match e {
                StripeError::CardDeclined => PaymentError::Rejected,
                StripeError::NetworkError => PaymentError::Network,
                _ => PaymentError::Network,
            })
    }
}
```

## Testing Strategy

### Unit Testing Domain Logic

Domain logic is easy to test with test doubles:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    struct MockProcessor {
        should_succeed: bool,
    }

    impl PaymentProcessor for MockProcessor {
        fn process(&self, _payment: &Payment) -> Result<(), PaymentError> {
            if self.should_succeed {
                Ok(())
            } else {
                Err(PaymentError::Rejected)
            }
        }
    }

    #[test]
    fn test_checkout_success() {
        let processor = MockProcessor { should_succeed: true };
        let service = CheckoutService::new(processor);

        assert!(service.checkout(5000, "USD").is_ok());
    }

    #[test]
    fn test_checkout_rejection() {
        let processor = MockProcessor { should_succeed: false };
        let service = CheckoutService::new(processor);

        assert!(service.checkout(5000, "USD").is_err());
    }
}
```

### Integration Testing Adapters

Test adapters against real systems (or test instances):

```rust
#[tokio::test]
async fn test_stripe_integration() {
    let processor = StripeProcessor {
        api_key: std::env::var("STRIPE_TEST_KEY").unwrap(),
    };

    let payment = Payment {
        amount_cents: 100,
        currency: "USD".to_string(),
    };

    let result = processor.process(&payment);
    assert!(result.is_ok());
}
```

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

For each dependency, create a trait:

```rust
// Before: tight coupling
struct UserService {
    db: PostgresConnection,
}

// After: trait extracted
trait UserRepository {
    fn find_by_id(&self, id: UserId) -> Result<User, Error>;
    fn save(&self, user: &User) -> Result<(), Error>;
}

struct UserService<R: UserRepository> {
    repo: R,
}
```

### Step 3: Implement Adapters

Create adapter structs that implement the traits:

```rust
struct PostgresUserRepository {
    pool: Pool<Postgres>,
}

impl UserRepository for PostgresUserRepository {
    fn find_by_id(&self, id: UserId) -> Result<User, Error> {
        // PostgreSQL implementation
    }

    fn save(&self, user: &User) -> Result<(), Error> {
        // PostgreSQL implementation
    }
}
```

### Step 4: Update Composition Root

Wire the new structure in main.rs:

```rust
fn main() {
    let pool = create_db_pool();
    let repo = PostgresUserRepository { pool };
    let service = UserService::new(repo);

    // Use service
}
```

## Common Mistakes to Avoid

### Mistake 1: Traits That Mirror Infrastructure APIs

```rust
// Bad: trait mirrors Stripe API
pub trait PaymentProcessor {
    fn create_charge(&self, token: &str, amount: u64) -> Result<ChargeId, Error>;
    fn refund_charge(&self, charge_id: &ChargeId) -> Result<(), Error>;
}

// Good: domain-focused trait
pub trait PaymentProcessor {
    fn process(&self, payment: &Payment) -> Result<(), PaymentError>;
    fn refund(&self, transaction: &TransactionId) -> Result<(), PaymentError>;
}
```

### Mistake 2: Domain Depending on Infrastructure Types

```rust
// Bad: domain uses infrastructure types
pub struct CheckoutService<P: PaymentProcessor> {
    processor: P,
}

impl<P: PaymentProcessor> CheckoutService<P> {
    pub fn checkout(&self, stripe_token: StripeToken) -> Result<(), Error> {
        // StripeToken is infrastructure detail
    }
}

// Good: domain uses domain types
impl<P: PaymentProcessor> CheckoutService<P> {
    pub fn checkout(&self, payment_method: PaymentMethod) -> Result<(), PaymentError> {
        // PaymentMethod is domain type
    }
}
```

### Mistake 3: Too Many Generic Parameters

```rust
// Bad: generic parameter explosion
pub struct OrderService<R: Repository, P: PaymentProcessor, E: EmailSender, L: Logger> {
    repo: R,
    payment: P,
    email: E,
    logger: L,
}

// Good: use trait objects or dedicated dependency struct
pub struct Dependencies {
    pub repo: Box<dyn Repository>,
    pub payment: Box<dyn PaymentProcessor>,
    pub email: Box<dyn EmailSender>,
    pub logger: Box<dyn Logger>,
}

pub struct OrderService {
    deps: Dependencies,
}
```

### Mistake 4: Putting Business Logic in Adapters

```rust
// Bad: business logic in adapter
impl PaymentProcessor for StripeProcessor {
    fn process(&self, payment: &Payment) -> Result<(), PaymentError> {
        // This validation is business logic - belongs in domain
        if payment.amount_cents < 100 {
            return Err(PaymentError::InvalidAmount);
        }

        self.stripe_client.charge(payment)
    }
}

// Good: business logic in domain
impl<P: PaymentProcessor> CheckoutService<P> {
    pub fn checkout(&self, amount_cents: u64, currency: &str) -> Result<(), PaymentError> {
        // Business rule in domain
        if amount_cents < 100 {
            return Err(PaymentError::InvalidAmount);
        }

        let payment = Payment { amount_cents, currency: currency.to_string() };
        self.processor.process(&payment)
    }
}
```

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
