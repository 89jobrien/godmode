# Mistake: domain depending on infrastructure types

Read this when a domain function signature mentions a type from an external SDK, such as a token or client struct. Domain signatures take domain types only.

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
