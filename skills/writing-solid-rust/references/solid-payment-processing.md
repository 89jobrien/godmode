# Domain: Payment Processing

## Minimal example

Domain: a `Payment` and a `PaymentProcessor` trait as the “port”. Concrete adapters for Stripe and a fake in‑memory processor.

```rust
// domain.rs (core domain, no external deps)

pub struct Payment {
    pub amount_cents: u64,
    pub currency: String,
}

#[derive(Debug)]
pub enum PaymentError {
    Rejected,
    Network,
}

pub trait PaymentProcessor {
    fn process(&self, payment: &Payment) -> Result<(), PaymentError>;
}

pub struct CheckoutService<P: PaymentProcessor> {
    processor: P,
}

impl<P: PaymentProcessor> CheckoutService<P> {
    pub fn new(processor: P) -> Self {
        Self { processor }
    }

    pub fn checkout(&self, amount_cents: u64, currency: &str) -> Result<(), PaymentError> {
        let payment = Payment {
            amount_cents,
            currency: currency.to_string(),
        };
        self.processor.process(&payment)
    }
}
```

```rust
// adapters/stripe.rs (infrastructure adapter)

use crate::domain::{Payment, PaymentError, PaymentProcessor};

pub struct StripeProcessor {
    pub api_key: String,
}

impl PaymentProcessor for StripeProcessor {
    fn process(&self, payment: &Payment) -> Result<(), PaymentError> {
        // Call Stripe API here; simplified for demo
        println!(
            "Stripe: charging {} {} using key {}",
            payment.amount_cents, payment.currency, self.api_key
        );
        Ok(())
    }
}
```

```rust
// adapters/in_memory.rs (test / dev adapter)

use crate::domain::{Payment, PaymentError, PaymentProcessor};

#[derive(Default)]
pub struct InMemoryProcessor {
    pub accepted: Vec<Payment>,
}

impl PaymentProcessor for InMemoryProcessor {
    fn process(&self, payment: &Payment) -> Result<(), PaymentError> {
        println!("InMemory: accepting {} {}", payment.amount_cents, payment.currency);
        // In real code, you'd use interior mutability (e.g. RefCell/Mutex) to store.
        Ok(())
    }
}
```

```rust
// main.rs (composition root)

mod domain;
mod adapters {
    pub mod stripe;
    pub mod in_memory;
}

use adapters::stripe::StripeProcessor;
use domain::CheckoutService;

fn main() {
    let processor = StripeProcessor {
        api_key: "sk_live_...".to_string(),
    };

    let checkout = CheckoutService::new(processor);

    checkout.checkout(5000, "USD").unwrap();
}
```

## Where SOLID shows up here

- **SRP**:
  - `Payment` is just data; `CheckoutService` owns business rules; each adapter only knows how to talk to its backend. [tuttlem.github](http://tuttlem.github.io/2025/08/31/hexagonal-architecture-in-rust.html)

- **OCP**:
  - You add new processors (PayPal, test double, etc.) via new `impl PaymentProcessor for X` without touching `CheckoutService` or the domain. [github](https://github.com/antoinecarton/hexagonal-rust)

- **DIP**:
  - Domain depends on the `PaymentProcessor` trait (abstraction), not Stripe or any concrete type; `main` wires concrete adapters. [antoniodiaz](https://www.antoniodiaz.me/en/blog/dependency-inversion-in-rust-6)
