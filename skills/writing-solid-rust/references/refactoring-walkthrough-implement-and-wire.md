# Refactoring walkthrough: implement adapters and update the composition root

Read this before implementing the adapters or rewiring the composition root in a trait-extraction refactor.

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
