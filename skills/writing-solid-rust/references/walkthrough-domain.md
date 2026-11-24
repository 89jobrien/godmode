# Payment walkthrough: domain port, types, and service

Read this before writing the domain port, domain types, or domain service for a port-and-adapter system. It holds the code for SKILL.md Steps 1 to 3, using the payment example.

## Step 1: Identify the Port

```rust
// domain.rs
pub trait PaymentProcessor {
    fn process(&self, payment: &Payment) -> Result<(), PaymentError>;
}
```

## Step 2: Define Domain Types

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

## Step 3: Implement Domain Logic

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
