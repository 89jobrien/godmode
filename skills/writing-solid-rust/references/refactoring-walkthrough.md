# Refactoring walkthrough: extract a trait, add an adapter, rewire

Read this when converting a tightly coupled service to ports and adapters. It holds the code for SKILL.md Refactoring Step 2, using a user repository. Steps 3 and 4 are in `refactoring-walkthrough-implement-and-wire.md`.

## Step 2: Extract Traits

```rust
// Before: tight coupling
struct UserService {
    db: PostgresConnection,
}

// After: trait extracted
trait UserRepository {
    fn find_by_id(&self, id: UserId) -> Result<User, Error>;
    fn save(&self, user: &User) -> Result<(), Error>;
}

struct UserService<R: UserRepository> {
    repo: R,
}
```
