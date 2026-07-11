---
name: "godmode:doc-sync"
description: >
  Detect drift between documentation and code. Use when docs may have fallen behind
  the implementation — stale CLI flags, removed modules, renamed types, outdated paths.
  Triggers on "sync docs", "docs are stale", "update docs", "check docs vs code".
requires: []
next: [doc-writer, doc-review, cap]
---

# Doc Sync

Find where documentation no longer matches the code. Code is always the source of truth.
This skill is read-only — it detects and reports drift; it does not fix.

## When to Use

- After a refactor, rename, or module restructure
- Before a release to catch stale references
- When a user reports incorrect documentation
- As part of the `gm:audit` workflow

## Drift Detection Checklist

### CLI surface

```
<binary> --help                    → compare against documented subcommands
<binary> <cmd> --help              → compare against documented flags per command
```

Check: every documented subcommand/flag exists in `--help`; every flag in `--help`
is documented (or explicitly marked internal).

### Crate / module surface

```
ls crates/                         → compare against documented crate list
grep -r '^pub mod' src/lib.rs      → compare against module table in CLAUDE.md/README
grep -r '^pub (fn|struct|enum|trait)' src/ → compare against API reference
```

### Skills and agents

```
ls skills/*/SKILL.md               → compare against skill tables in README/CLAUDE.md
ls agents/*.md                     → compare against agents/INDEX.md and README agents table
```

### File paths

Extract every literal path from all docs (`README.md`, `CLAUDE.md`, `docs/**/*.md`).
For each path: check it exists. Flag missing paths as **Blocking**.

### Cross-doc consistency

- Same feature described in multiple docs → descriptions must agree
- Skill `name:` in frontmatter must match its entry in any skill index
- Agent `name:` must match its entry in `agents/INDEX.md`

## Output Format

Group findings by severity:

**Blocking** — would cause a user to fail following the docs:

- References to removed CLI flags or subcommands
- Paths that no longer exist
- Instructions that would produce errors

**Suggestion** — docs are incomplete or misleading:

- Undocumented features that exist in code
- Descriptions that don't match current behaviour
- Missing table rows for new crates/skills/agents

**Nitpick** — minor drift:

- Wording differences between docs that describe the same thing
- Outdated version numbers or dates

## After Reporting

- For each Blocking finding: suggest the minimal edit needed
- Hand off to `godmode:doc-writer` for missing docs
- Hand off to `godmode:doc-review` after fixes are applied
- Do not apply fixes yourself — this skill is read-only
