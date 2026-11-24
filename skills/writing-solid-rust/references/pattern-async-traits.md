# Pattern: async traits

Read this when a port trait has async methods, such as an LLM provider or an HTTP-backed service.

```rust
use async_trait::async_trait;

#[async_trait]
pub trait LlmProvider {
    async fn chat(&self, messages: &[ChatMessage]) -> Result<String, LlmError>;
}

#[async_trait]
impl LlmProvider for OpenAiAdapter {
    async fn chat(&self, messages: &[ChatMessage]) -> Result<String, LlmError> {
        // Async implementation
    }
}
```
