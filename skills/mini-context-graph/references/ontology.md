# Ontology — Entity and Relation Types

## Entity Types

| Type           | Description                  | Examples                                |
| -------------- | ---------------------------- | --------------------------------------- |
| `person`       | Named individual             | "Linus Torvalds", "Joe"                 |
| `organization` | Company, team, group         | "Anthropic", "IETF"                     |
| `system`       | Software system or service   | "minibox", "PostgreSQL"                 |
| `concept`      | Abstract idea or pattern     | "hexagonal architecture", "trust decay" |
| `issue`        | Bug, vulnerability, problem  | "memory leak", "race condition"         |
| `tool`         | CLI tool, library, framework | "cargo", "tokio", "kgx"                 |
| `standard`     | Spec, RFC, protocol          | "HTTP/2", "RFC 7519"                    |
| `file`         | Specific file or path        | "src/lib.rs", "Cargo.toml"              |
| `event`        | Incident, release, milestone | "v2.0 release", "outage 2024-03"        |

## Relation Types

| Type             | Direction       | Description                                  |
| ---------------- | --------------- | -------------------------------------------- |
| `causes`         | source → target | source causes or triggers target             |
| `depends_on`     | source → target | source requires target                       |
| `implements`     | source → target | source implements target (trait, spec)       |
| `contains`       | source → target | source contains or owns target               |
| `authored_by`    | source → target | source was created by target                 |
| `uses`           | source → target | source uses or calls target                  |
| `conflicts_with` | bidirectional   | source and target are incompatible           |
| `supersedes`     | source → target | source replaces target                       |
| `related_to`     | bidirectional   | generic; use only when no specific type fits |

## Normalization Rules

- Entity names: lowercase, spaces (not underscores or hyphens).
- Deduplicate: "Memory Leak" and "memory leak" are the same entity.
- Acronyms: expand on first use, then use the short form as canonical name.
- Versions: include version in name only when the version matters
  ("rust 1.82" vs just "rust").
