# Mistake: business logic in adapters

Read this when an adapter method contains a validation or a business rule. Rules belong in the domain service, so they run regardless of which adapter is wired in.

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
