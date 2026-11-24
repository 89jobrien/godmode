# Reference examples

Read this before choosing one of the three full domain examples to copy from: LLM inference, UI components, or payment processing.

## Reference Examples

This skill includes three complete domain examples in the `references/` directory:

### 1. LLM Inference (`references/solid-LLM-inference.md`)

Demonstrates SOLID principles for AI/LLM integration:

- **Domain**: `LlmProvider` trait as port
- **Components**: `CodeReviewer` and `QaAgent` generic over providers
- **Adapters**: OpenAI, Claude (Anthropic), and Ollama implementations
- **Pattern**: Easily swap LLM providers without changing business logic

### 2. UI Components (`references/solid-UI-components.md`)

Shows render-agnostic UI component architecture:

- **Domain**: `Component` and `Renderer` traits
- **Components**: `Label`, `Button`, `LoginForm`
- **Adapters**: Terminal and Web renderers
- **Pattern**: Same UI components render to multiple backends

### 3. Payment Processing (`references/solid-payment-processing.md`)

Minimal payment processing example:

- **Domain**: `PaymentProcessor` trait
- **Components**: `CheckoutService` with business rules
- **Adapters**: Stripe and InMemory implementations
- **Pattern**: Swap payment processors, use test doubles easily

## Additional Resources

For complete working examples, see:

- `references/solid-LLM-inference.md` - AI/LLM port and adapters
- `references/solid-UI-components.md` - Render-agnostic UI components
- `references/solid-payment-processing.md` - Payment processing example

Each reference includes full code examples with explanations of where SOLID principles appear.
