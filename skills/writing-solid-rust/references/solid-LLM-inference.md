# Domain: LLM port

Here’s the pattern adapted for AI/LLM work: a domain-level `LLM` trait as the port, concrete agent components, and adapters for OpenAI, Claude (Anthropic), and a local Ollama backend. [dev](https://dev.to/leapcell/trait-in-rust-explained-from-basics-to-advanced-usage-14mn)

## Code

### ai/domain.rs

```rust
use async_trait::async_trait;
use serde_json::Value;

#[derive(Debug)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
}

#[derive(Debug, Clone)]
pub enum Role {
    User,
    Assistant,
    System,
}

#[derive(Debug)]
pub enum LlmError {
    Api(String),
    Parse,
}

#[async_trait]
pub trait LlmProvider {
    async fn chat(&self, messages: &[ChatMessage]) -> Result<String, LlmError>;
    async fn complete(&self, prompt: &str) -> Result<String, LlmError>;
}
```

## Domain components

### ai/agents.rs

```rust
use crate::ai::domain::{LlmProvider, ChatMessage, Role, LlmError};

pub struct CodeReviewer<P: LlmProvider> {
    provider: P,
    context: Vec<ChatMessage>,
}

impl<P: LlmProvider> CodeReviewer<P> {
    pub fn new(provider: P) -> Self {
        let mut context = vec![];
        context.push(ChatMessage {
            role: Role::System,
            content: "You are a senior Rust engineer reviewing code.".into(),
        });
        Self { provider, context }
    }

    pub async fn review(&mut self, code: &str) -> Result<String, LlmError> {
        self.context.push(ChatMessage {
            role: Role::User,
            content: format!("Review this Rust code:\n{}", code),
        });
        let response = self.provider.chat(&self.context).await?;
        self.context.push(ChatMessage {
            role: Role::Assistant,
            content: response.clone(),
        });
        Ok(response)
    }
}

pub struct QaAgent<P: LlmProvider> {
    provider: P,
}

impl<P: LlmProvider> QaAgent<P> {
    pub fn new(provider: P) -> Self {
        Self { provider }
    }

    pub async fn answer(&self, question: &str) -> Result<String, LlmError> {
        let messages = vec![ChatMessage {
            role: Role::User,
            content: question.into(),
        }];
        self.provider.chat(&messages).await
    }
}
```

## Adapters

### infra/openai.rs

```rust
// simplified; use async reqwest + tokio in real code
use crate::ai::domain::{LlmProvider, ChatMessage, Role, LlmError};
use async_trait::async_trait;
use reqwest::Client;

pub struct OpenAiAdapter {
    client: Client,
    api_key: String,
    model: String,
}

impl OpenAiAdapter {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model: "gpt-4o-mini".into(),
        }
    }
}

#[async_trait]
impl LlmProvider for OpenAiAdapter {
    async fn chat(&self, messages: &[ChatMessage]) -> Result<String, LlmError> {
        // Real impl would POST to https://api.openai.com/v1/chat/completions
        // with JSON payload including model, messages, etc.
        // Parse response.content[0].text or similar.
        println!("OpenAI chat: {:?} -> simulating response", messages.last());
        Ok("OpenAI says: Code looks solid, but add more tests.".into())
    }

    async fn complete(&self, prompt: &str) -> Result<String, LlmError> {
        println!("OpenAI complete: {} -> simulating", prompt);
        Ok("fn main() { println!(\"Hello\"); }".into())
    }
}
```

### infra/claude.rs

```rust
use crate::ai::domain::{LlmProvider, ChatMessage, Role, LlmError};
use async_trait::async_trait;

pub struct ClaudeAdapter {
    api_key: String,
}

impl ClaudeAdapter {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

#[async_trait]
impl LlmProvider for ClaudeAdapter {
    async fn chat(&self, messages: &[ChatMessage]) -> Result<String, LlmError> {
        println!("Claude chat: {:?} -> simulating", messages.last());
        Ok("Claude says: Great architecture! Traits for DIP are perfect.".into())
    }

    async fn complete(&self, _prompt: &str) -> Result<String, LlmError> {
        Ok("Claude completion simulation.".into())
    }
}
```

### infra/ollama.rs

```rust
use crate::ai::domain::{LlmProvider, ChatMessage, Role, LlmError};
use async_trait::async_trait;

pub struct OllamaAdapter {
    model: String,
}

impl OllamaAdapter {
    pub fn new(model: String) -> Self {
        Self { model }
    }
}

#[async_trait]
impl LlmProvider for OllamaAdapter {
    async fn chat(&self, messages: &[ChatMessage]) -> Result<String, LlmError> {
        println!("Ollama {} chat: {:?} -> simulating", self.model, messages.last());
        Ok("Local model: Ownership issues? Use Arc<Mutex<State>>.".into())
    }

    async fn complete(&self, _prompt: &str) -> Result<String, LlmError> {
        Ok("Ollama completion.".into())
    }
}
```

## Composition root

### main.rs

```rust
mod ai {
    pub mod domain;
    pub mod agents;
}
mod infra {
    pub mod openai;
    pub mod claude;
    pub mod ollama;
}

use ai::agents::{CodeReviewer, QaAgent};
use ai::domain::LlmError;
use infra::{OpenAiAdapter, ClaudeAdapter, OllamaAdapter};

#[tokio::main]
async fn main() -> Result<(), LlmError> {
    // Wire OpenAI for code review
    let openai = OpenAiAdapter::new("sk-...".into());
    let mut reviewer = CodeReviewer::new(openai);
    let review = reviewer.review("fn foo() {}").await?;
    println!("Review: {}", review);

    // Wire Claude for Q&A
    let claude = ClaudeAdapter::new("anth-...".into());
    let qa = QaAgent::new(claude);
    let answer = qa.answer("Best Rust UI crate?").await?;
    println!("Q&A: {}", answer);

    // Swap to local Ollama
    let ollama = OllamaAdapter::new("llama3.1".into());
    let qa_local = QaAgent::new(ollama);
    let local_answer = qa_local.answer("SOLID in Rust?").await?;
    println!("Local Q&A: {}", local_answer);

    Ok(())
}
```

---

## Where SOLID shows up

- **SRP**
  - `CodeReviewer` only reviews code (business rule); adapters only handle API calls; no leakage. [github](https://github.com/graniet/llm)

- **OCP**
  - New agents: `impl` via new generics. New providers: new `impl LlmProvider` (e.g., Grok, Gemini). [lib](https://lib.rs/crates/llmao)

- **DIP**
  - Domain depends on `LlmProvider` trait; `main` injects concrete adapters. Test with mocks easily. [github](https://github.com/graniet/llm)
