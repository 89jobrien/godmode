# Testing: integration tests for adapters

Read this when testing an adapter against a real system or a test instance. These tests are slower than unit tests and need credentials, so keep them out of the default unit run.

```rust
#[tokio::test]
async fn test_stripe_integration() {
    let processor = StripeProcessor {
        api_key: std::env::var("STRIPE_TEST_KEY").unwrap(),
    };

    let payment = Payment {
        amount_cents: 100,
        currency: "USD".to_string(),
    };

    let result = processor.process(&payment);
    assert!(result.is_ok());
}
```
