# Pattern: multiple traits for different capabilities

Read this when one trait has grown several methods that different callers never all use. Split it by capability (ISP) so each component depends only on what it needs.

```rust
// Instead of one large trait
pub trait Storage {
    fn read(&self, key: &str) -> Result<Vec<u8>, Error>;
    fn write(&self, key: &str, data: &[u8]) -> Result<(), Error>;
    fn delete(&self, key: &str) -> Result<(), Error>;
    fn list(&self) -> Result<Vec<String>, Error>;
}

// Use separate traits (ISP)
pub trait Reader {
    fn read(&self, key: &str) -> Result<Vec<u8>, Error>;
}

pub trait Writer {
    fn write(&self, key: &str, data: &[u8]) -> Result<(), Error>;
}

pub trait Deleter {
    fn delete(&self, key: &str) -> Result<(), Error>;
}

// Components depend only on what they need
pub struct ReadOnlyService<R: Reader> {
    reader: R,
}
```
