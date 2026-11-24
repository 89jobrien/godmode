# Refactoring existing code: steps

Read this before extracting traits from an existing service, to see the four steps in order. The code for each step is in the refactoring rows of the index in SKILL.md.

## Refactoring Existing Code

### Step 1: Identify External Dependencies

Find all direct usage of:

- HTTP clients
- Database connections
- File system operations
- Third-party APIs

### Step 2: Extract Traits

For each dependency, create a trait. Code: `references/refactoring-walkthrough.md`.

### Step 3: Implement Adapters

Create adapter structs that implement the traits. Code: `references/refactoring-walkthrough-implement-and-wire.md`.

### Step 4: Update Composition Root

Wire the new structure in main.rs. Code: `references/refactoring-walkthrough-implement-and-wire.md`.
