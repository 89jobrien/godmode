# Issue Template for TODO Sync

## GitHub Issue Format

**Title**: `fix/feat(<module>): <todo text>`

**Body**:

```markdown
## Context

`<file>:<line>` has an unresolved TODO.

## TODO
```

<full TODO comment text>
```

## Surrounding Code

```rust
<5 lines before>
// TODO: <the todo>
<5 lines after>
```

## Action

Resolve or implement the TODO at `<file>:<line>`.

````

**Labels**: infer from directory path:
- `crates/godmode-core/` → `core`
- `crates/godmode-cli/` → `cli`
- `hooks/` → `hooks`
- `skills/` → `skills`

## Decision: fix vs feat

- `fix` — the TODO describes a bug, missing error handling, or broken behaviour
- `feat` — the TODO describes unimplemented functionality or an enhancement

## Inline Annotation Format

After creating the issue, annotate the TODO:
```rust
// TODO(#42): implement rate limiting
````
