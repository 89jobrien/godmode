# Testing: unit tests for domain logic

Read this when writing unit tests for domain services. Test doubles implement the port trait directly, so no external system is involved.

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
