---
name: "godmode:doc-maintainer"
description: >
  Audit and maintain project documentation. Compares CLAUDE.md, README.md,
  agents/INDEX.md, and skill descriptions against actual source code. Detects
  stale references, undocumented features, and cross-doc inconsistencies.
  Use when asked to "check docs", "audit documentation", or "sync docs".
requires: []
next: [cap]
---

# Doc Maintainer

Keep documentation in sync with code. The code is always the source of truth.

## When to Use

- After adding or removing CLI subcommands
- After adding or removing core modules
- After adding or removing skills or agents
- Before a release
- When docs feel stale or a user reports incorrect instructions

## Three-Check Audit

### 1. Stale references

Docs mention something that no longer exists in the codebase.

**How to detect:**

- Extract every CLI subcommand from `godmode --help`; compare against the
  subcommand table in CLAUDE.md
- Extract every `pub mod` from `crates/godmode-core/src/lib.rs`; compare
  against the module table in CLAUDE.md
- List `agents/*.yaml`; compare against `agents/INDEX.md`
- List `skills/*/SKILL.md`; compare against any skill inventories

### 2. Undocumented features

Code has something that docs don't mention.

**How to detect:**

- Same comparisons as above, reversed — look for items in code not in docs
- Check `godmode <cmd> --help` flags against documented flag lists
- Check for new `pub fn` or `pub struct` exports not mentioned in
  architecture docs

### 3. Cross-doc inconsistencies

Two documents describe the same thing differently.

**How to detect:**

- Compare feature descriptions between CLAUDE.md and README.md
- Compare skill SKILL.md descriptions against their agent YAML descriptions
- Compare module table entries against actual `//!` doc comments

## Output Format

Prioritized finding list:

- **Blocking**: broken paths, references to removed features, incorrect
  instructions that would cause errors
- **Suggestion**: undocumented features, missing table rows, incomplete
  descriptions
- **Nitpick**: wording differences, minor inconsistencies, style issues

## Fix Mode

When asked to fix (not just audit):

1. Fix blocking issues first
2. Use Edit for targeted changes — do not rewrite whole files
3. For `agents/INDEX.md`, prefer regenerating via `godmode agent index`;
   hand-edit only if the CLI output needs a targeted correction
4. Re-audit after fixing to confirm zero blocking findings
5. Make minimum edits — do not restructure or restyle documents
