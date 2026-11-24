# Payment walkthrough: adapters and composition root

Read this before writing infrastructure adapters or wiring them in `main.rs`. It holds the code for SKILL.md Steps 4 and 5, using the payment example. Step 3 is in `walkthrough-domain.md`.

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

## Step 5: Wire Dependencies in Composition Root

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
