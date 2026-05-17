# Split Strategies — Decision Guide

Reference for LLM judgment calls during decomposition. The mechanical analysis handles
file→crate mapping and concern classification. This guide covers the ambiguous cases.

## When to merge two proposed splits

Merge splits when they are semantically coupled even if mechanically separate:

| Signal                                                        | Action                                                                              |
| ------------------------------------------------------------- | ----------------------------------------------------------------------------------- |
| Split A introduces a trait; Split B implements it             | Merge — impl without trait definition won't compile                                 |
| Split A renames a type; Split B uses it                       | Merge — or put the rename in a prep PR first                                        |
| Split A changes a public function signature; Split B calls it | Merge — or split as: [old signature] then [callers] then [new signature]            |
| Both splits change the same `mod` or `pub use` re-export      | Merge — export list must be consistent                                              |
| Coupling warning shows shared `use crate::` path              | Review — may be fine (read-only shared type) or blocking (mutation of shared state) |

## When to keep splits separate despite coupling warnings

| Signal                                                           | Action                                        |
| ---------------------------------------------------------------- | --------------------------------------------- |
| Shared import is a pure data type (struct with no behavior)      | Keep separate — the type won't change         |
| Shared import is a trait from an external crate                  | Keep separate — not affected by local changes |
| Files share an import but modify completely different code paths | Keep separate — coupling is nominal, not real |

## Split ordering for dependent splits

When splits cannot be fully independent, establish an ordering:

1. **Foundation first**: deps bumps, trait definitions, new modules go in split 1
2. **Implementations next**: impls of the new traits, callers of the new APIs go in split 2
3. **Tests last**: test coverage for the new behavior goes in split 3

Document the ordering in each PR body: "Depends on #<prior-PR>".

## Naming convention

```
<source-branch>-split-<N>-<concern>[-<crate>]
```

Examples:

- `feat/auth-split-1-deps` — Cargo.toml / dependency bumps
- `feat/auth-split-2-core` — core logic in `auth` crate
- `feat/auth-split-3-tests` — test additions

For workspace-wide changes with no dominant crate, omit the crate suffix:

- `refactor/rename-split-1-logic`
- `refactor/rename-split-2-tests`

## Concern priorities (when a file fits multiple concerns)

A file matches at most one concern — first match wins:

1. `deps` — Cargo.toml / Cargo.lock (always separate)
2. `ci` — .github/ (always separate)
3. `tests` — /tests/ paths
4. `benches` — /benches/ paths
5. `docs` — .md / docs/ paths
6. `examples` — /examples/ paths
7. `scripts` — .nu / .sh files
8. `logic` — everything else

## Size heuristics

| Split size  | Assessment                                                                  |
| ----------- | --------------------------------------------------------------------------- |
| 1–5 files   | Good — very focused                                                         |
| 6–15 files  | Acceptable — confirm it's a single concern                                  |
| 16–30 files | Large — look for a sub-concern to extract                                   |
| 30+ files   | Too large — split further unless it's a mechanical rename across many files |

A mechanical rename (e.g., `s/OldType/NewType/` across 50 files) is acceptable as one split
because it is trivially reviewable despite its size.

## The "prep PR" pattern

When a large changeset cannot be cleanly split (e.g., a refactor that touches every call site),
use a prep PR:

1. **Prep PR**: add the new API alongside the old (no behavior change, no callers updated)
2. **Migration PR(s)**: update callers crate by crate
3. **Cleanup PR**: remove the old API

This pattern produces independently mergeable PRs even when the end state requires all three.
