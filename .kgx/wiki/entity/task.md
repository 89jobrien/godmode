# Task

Core data type representing a unit of work in the task graph.

## Fields

- `id: String` — unique identifier (e.g. "t1", "t2")
- `title: String` — human-readable description
- `status: Status` — Pending | Running | Done | Blocked
- `depends_on: Vec<String>` — IDs of prerequisite tasks
- `crate_name: Option<String>` — target crate for dispatch grouping
- `commit: Option<String>` — SHA recorded on completion
- `run: Option<String>` — shell command; prefix `rx:` for rx registry
- `started_at / completed_at: Option<DateTime<Utc>>` — timing bounds
- `priority: Priority` — High | Normal (default, skipped in YAML) | Low
- `tags: Vec<String>` — freeform grouping

## Defined in

`crates/godmode-core/src/model.rs`

## Key behaviors

- `started_at` is set by `Session::start_task`, not `graph::start`
- Normal priority is omitted from YAML via `skip_serializing_if`
- Empty tags and depends_on are omitted from YAML

## Related

- [[TaskGraph]] — container for tasks
- [[Session]] — orchestrates state transitions
- [[Chain]] — dispatch grouping by crate
