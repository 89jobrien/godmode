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
