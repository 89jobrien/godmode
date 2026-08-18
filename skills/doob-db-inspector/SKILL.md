---
name: doob-db-inspector
description: Inspects and reports on the live doob SurrealDB database — todo health, overdue items, stale in-progress, orphaned todos, cross-project drift, and tag distribution. Use when auditing database state or diagnosing data quality issues in doob.
---

# doob DB Inspector

Audits the live doob database and produces a health report.

## Database Location

`~/.claude/data/doob.db` (SurrealDB with RocksDB backend)

## Inspection via doob CLI

Query the database using the doob CLI — no direct DB access needed:

```bash
doob todo list --json
doob note list --json
doob todo list --status pending --json
doob todo list --status in_progress --json
doob todo list --status completed --json
doob todo list --status cancelled --json
```

## Report Sections

1. **Summary** — total todos by status, total notes, projects represented.
2. **Overdue** — todos with `due_date` < today and status != completed/cancelled.
   Format: `[id] content (due: date, priority: N)`.
3. **Stale In-Progress** — `in_progress` todos with `updated_at` > 7 days ago; likely abandoned,
   suggest `doob todo undo <id>`.
4. **High-Priority Pending** — pending todos with `priority` > 150; blocking work.
5. **Orphaned Todos** — todos with a `project` field set but `project_path` null/empty; may
   indicate moved/deleted repos.
6. **Tag Distribution** — count todos per tag; flag tags with only 1 todo as possible typos.

## Output Format

Use a human-readable markdown table per section. If a section is clean, write "None found."
