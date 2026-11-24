# Pattern: builder for complex adapters

Read this when an adapter needs several configuration values, some of them required.

```rust
pub struct OpenAiAdapter {
    client: Client,
    api_key: String,
    model: String,
    timeout: Duration,
}

impl OpenAiAdapter {
    pub fn builder() -> OpenAiAdapterBuilder {
        OpenAiAdapterBuilder::default()
    }
}

pub struct OpenAiAdapterBuilder {
    api_key: Option<String>,
    model: String,
    timeout: Duration,
}

impl OpenAiAdapterBuilder {
    pub fn api_key(mut self, key: String) -> Self {
        self.api_key = Some(key);
        self
    }

    pub fn model(mut self, model: String) -> Self {
        self.model = model;
        self
    }

    pub fn build(self) -> Result<OpenAiAdapter, BuildError> {
        Ok(OpenAiAdapter {
            client: Client::new(),
            api_key: self.api_key.ok_or(BuildError::MissingApiKey)?,
            model: self.model,
            timeout: self.timeout,
        })
    }
}
```
