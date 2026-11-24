# Payment walkthrough: composition root

Read this before wiring the adapters into a composition root for the payment example.

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
