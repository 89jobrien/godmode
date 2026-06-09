---
name: "godmode:design"
description: >
  Translate an approved brainstorm into a precise architectural specification. Defines crate
  ownership, public API (traits, types, signatures), data flow, and hexagonal port boundaries.
  Triggers after brainstorm approval, before writing-plans.
requires: [brainstorm, context-map]
next: [writing-plans]
---

# Design

**Hard gate**: Do NOT write any code or invoke `godmode:writing-plans` until the design doc
is written and the user has explicitly approved it. No exceptions.

## When to Run

- After `godmode:brainstorm` produces an approved approach
- When given an architectural decision that needs formal specification before planning

## Process

### Step 1: Read the brainstorm output

Confirm you have:

- An approved approach (one of the proposed options, not still open)
- The goal statement in one sentence
- Any explicit constraints or out-of-scope items

If any of these are missing, run `godmode:brainstorm` first.

### Step 2: Run context-map

Run `godmode:context-map` before writing anything. The context map determines:

- Which crate owns the new code (never guess)
- Which existing types or traits the design must extend or compose
- Which files will be affected
- Risk flags (API surface changes, semver implications)

### Step 3: Define the architecture

For each design, specify:

**Crate ownership**

- Which existing crate, or does this require a new crate?
- If new: what is its single responsibility, and why doesn't it belong in an existing crate?

**Public API**

- Every new trait: name, methods, associated types
- Every new type: name, fields (named, not unnamed tuples), derives
- Every new public function: signature (no bodies — signatures only)
- No `todo!()`, no `unimplemented!()`, no placeholders

**Data flow**

- Source → transform → sink, one sentence per hop
- Identify the hexagonal boundary: what is a port (trait), what is an adapter (impl)?

**Integration points**

- How does new code connect to existing modules?
- Does it go behind a feature flag?
- Does it change any existing public API? If yes, enumerate breaking changes.

### Step 4: Apply doublecheck

Before writing the design doc, verify:

- [ ] Every new type is placed in the correct crate per the context map
- [ ] No circular dependencies are introduced
- [ ] The API surface is minimal — remove anything not required by the current goal
- [ ] External dependencies, if any, are behind a trait (hexagonal rule)
- [ ] Names are consistent with existing conventions in the codebase

If doublecheck surfaces issues, resolve them before proceeding.

### Step 5: Write the design document

Save to: `docs/designs/YYYY-MM-DD-<feature-name>-design.md`

````markdown
# Design: <Feature Name>

## Goal

One sentence. What does this implement and why.

## Approved Approach

State which approach from brainstorm was selected, in one sentence.

## Crate Ownership

- **Owner crate**: `<crate-name>` — reason in one sentence
- **Affected crates**: list any crates that will import the new code

## Public API

### Traits

```rust
pub trait <Name> {
    fn <method>(&self, ...) -> ...;
}
```
````

### Types

```rust
pub struct <Name> {
    pub field: Type,
}
```

### Functions

```rust
pub fn <name>(...) -> ...;
```

## Data Flow

1. Source: ...
2. Transform: ...
3. Sink: ...

## Hexagonal Boundaries

- **Port** (trait): `<TraitName>` in `<crate>::<module>`
- **Adapter** (impl): `<ImplName>` in `<crate>::<module>`

## Out of Scope

- List anything explicitly excluded from this design

## Risk

- [ ] Breaking API changes: yes/no — if yes, list them
- [ ] New external dependency: yes/no — if yes, name and justify
- [ ] Feature flag required: yes/no

```

### Step 6: Get user approval

Present the design doc. Do not proceed until the user explicitly approves.

### Step 7: Invoke `godmode:writing-plans`

After approval, hand off to writing-plans. The plan's task list must reference exact file
paths and type names from this design doc — no invention allowed.

## Design Rules

- **Signatures only**: the design doc contains signatures, not implementations
- **No placeholders**: every type name, method name, and field name must be final
- **Hexagonal by default**: external I/O, processes, and storage go behind traits
- **One crate, one responsibility**: if the design spans more than 3 crates, it is too large
- **Consistent names**: check existing code for naming conventions before coining new names

## Additional Resources

- `references/design-principles.md` — hexagonal architecture, YAGNI, naming conventions
- `helpers/design-doc-template.md` — blank design doc
```
