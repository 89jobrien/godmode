# Hexagonal architecture

Read this before drawing the hexagonal layout for a new system, or before deciding which layer a type or trait belongs to.

### Hexagonal Architecture

```text
┌─────────────────────────────────────────────────┐
│              Composition Root                   │
│                  (main.rs)                      │
│   Wires domain to adapters, injects deps        │
└────────────────┬────────────────────────────────┘
                 │
    ┌────────────┴────────────┐
    │                         │
┌───▼─────────────┐    ┌──────▼──────────────┐
│   Domain Layer  │    │ Infrastructure      │
│                 │    │    Adapters         │
│  ┌──────────┐   │    │                     │
│  │Business  │   │    │  ┌──────────────┐   │
│  │Logic     │───┼────┼─►│ OpenAI       │   │
│  │          │   │    │  │ Adapter      │   │
│  └──────────┘   │    │  └──────────────┘   │
│                 │    │                     │
│  ┌──────────┐   │    │  ┌──────────────┐   │
│  │Traits    │◄──┼────┼──│ Stripe       │   │
│  │(Ports)   │   │    │  │ Adapter      │   │
│  └──────────┘   │    │  └──────────────┘   │
│                 │    │                     │
│  ┌──────────┐   │    │  ┌──────────────┐   │
│  │Domain    │   │    │  │ Terminal     │   │
│  │Types     │   │    │  │ Renderer     │   │
│  └──────────┘   │    │  └──────────────┘   │
└─────────────────┘    └─────────────────────┘

Dependencies point inward: adapters → domain
Domain has zero dependencies on infrastructure
</text>
```
