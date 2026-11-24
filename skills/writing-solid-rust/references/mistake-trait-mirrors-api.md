# Mistake: traits that mirror infrastructure APIs

Read this when designing a port from an external API's method list. Name the trait after the domain operation, not after the vendor's endpoints.

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
