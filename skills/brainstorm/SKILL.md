---
name: "godmode:brainstorm"
description: >
  Use before any creative work — new features, new components, architecture decisions,
  adding crates, designing APIs. Triggers on "let's build", "I want to add", "design X",
  "how should we structure", or any request that would produce new code structure.
requires: []
next: [design]
---

# Brainstorm

**Hard gate**: Do NOT write any code, scaffold any structure, or invoke any implementation
skill until you have presented a design and the user has explicitly approved it. No
exceptions — including "simple" features.

## Process

### Step 1: Explore context

Before asking anything, read:

- The relevant `CLAUDE.md` for the project
- The crate's `Cargo.toml` and entry points
- Any existing analogous code

### Step 2: Ask one clarifying question at a time

Never dump a list of questions. One question → user answers → next question.
Prefer multiple-choice over open-ended where possible.

### Step 3: Propose 2-3 approaches

For each approach:

- Name it
- Describe it in 2-3 sentences
- State the key trade-off vs. the alternatives

### Step 4: Present design sections iteratively

Break the design into sections. Present one section → get feedback → refine → next section.
Do not present the entire design at once.

### Step 5: Invoke `godmode:design`

Once the user has approved an approach, hand off to `godmode:design`. Brainstorm does not
write code, types, or implementation plans — that belongs to the design and planning stages.

Pass forward:

- The approved approach (named)
- The goal statement
- Any explicit constraints or out-of-scope items stated during brainstorming

## Design Principles

- **Hexagonal architecture**: new external dependencies behind a trait (port), implementations
  in adapters. Business logic is generic over trait bounds.
- **YAGNI**: ruthlessly remove anything not needed for the current requirement.
- **Rust idioms**: prefer `Result`/`Option` over panics, `impl Trait` over `Box<dyn Trait>`
  where lifetimes allow, `#[derive]` over manual impls.
- **One question at a time**: overwhelming the user with 5 questions at once is a failure mode.

## Rust-Specific Questions to Consider

- Which crate owns this? Does it belong in an existing crate or needs a new one?
- Does this introduce a new external dependency? Can it be behind a feature flag?
- Will this be tested with in-memory fakes or does it need integration tests?
- Does this change the public API surface? Semver implications?

## Additional Resources

- **`references/design-principles.md`** — hexagonal architecture, YAGNI, Rust idioms reference
- **`helpers/design-doc-template.md`** — blank design doc to fill in during Step 5
