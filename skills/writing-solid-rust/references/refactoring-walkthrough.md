# Refactoring walkthrough: extract a trait, add an adapter, rewire

Read this when converting a tightly coupled service to ports and adapters. It holds the code for SKILL.md Refactoring Steps 2 to 4, using a user repository.

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

## Step 3: Implement Adapters

```rust
struct PostgresUserRepository {
    pool: Pool<Postgres>,
}

impl UserRepository for PostgresUserRepository {
    fn find_by_id(&self, id: UserId) -> Result<User, Error> {
        // PostgreSQL implementation
    }

    fn save(&self, user: &User) -> Result<(), Error> {
        // PostgreSQL implementation
    }
}
```

## Step 4: Update Composition Root

```rust
fn main() {
    let pool = create_db_pool();
    let repo = PostgresUserRepository { pool };
    let service = UserService::new(repo);

    // Use service
}
```
