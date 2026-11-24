# Payment walkthrough: adapters and composition root

Read this before writing infrastructure adapters for the payment example. It holds the code for SKILL.md Step 4. Step 3 is in `walkthrough-domain.md`. Step 5 is in `walkthrough-composition-root.md`.

## Step 4: Create Infrastructure Adapters

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

Test double:

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
