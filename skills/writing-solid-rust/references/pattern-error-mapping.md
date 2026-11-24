# Pattern: error type mapping

Read this when an adapter calls an external system whose error types would otherwise leak into the domain. The adapter maps each infrastructure error to a domain error.

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
