# Mistake: too many generic parameters

Read this when a service struct has four or more generic parameters. Group the dependencies into one struct, or use trait objects, so the service signature stays readable.

```rust
// Bad: generic parameter explosion
pub struct OrderService<R: Repository, P: PaymentProcessor, E: EmailSender, L: Logger> {
    repo: R,
    payment: P,
    email: E,
    logger: L,
}

// Good: use trait objects or dedicated dependency struct
pub struct Dependencies {
    pub repo: Box<dyn Repository>,
    pub payment: Box<dyn PaymentProcessor>,
    pub email: Box<dyn EmailSender>,
    pub logger: Box<dyn Logger>,
}

pub struct OrderService {
    deps: Dependencies,
}
```
