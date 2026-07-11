---
name: "godmode:doc-review"
description: >
  Review documentation for accuracy, completeness, and clarity before publishing.
  Use after writing or updating docs, before a release, or when asked to "review docs",
  "proofread", or "check this doc". Read-only — flags issues, does not rewrite.
requires: []
next: [cap]
---

# Doc Review

Structured review of documentation before it ships. Covers accuracy, completeness,
clarity, and navigability. Read-only — report findings; do not rewrite the doc.

## When to Use

- After `godmode:doc-writer` produces a new doc
- After `godmode:doc-sync` fixes are applied
- Before tagging a release
- When asked to proofread or review documentation

## Review Dimensions

### 1. Accuracy

Every claim in the doc must be verifiable against the code or CLI output.

- Run every command shown — confirm it produces the documented output
- Check every file path — confirm it exists
- Check every flag and option — confirm it appears in `--help`
- Check every type, function, or module name — confirm it exists in source
- Flag any claim you cannot verify as **Blocking**

### 2. Completeness

The doc covers what a reader needs to accomplish the documented task without
having to read source code.

- Install / setup: is everything needed to get started present?
- CLI reference: are all subcommands and their flags documented?
- Architecture doc: are all components and their ownership described?
- API reference: does every public export have a purpose and example?
- Flag anything missing as **Suggestion**

### 3. Clarity

A reader encountering this for the first time can follow it without confusion.

- Is the entry point (quickstart, install) obvious?
- Are terms defined before they are used?
- Are examples concrete (real commands, real output) rather than abstract?
- Are tables used where a list of items has multiple attributes?
- Flag anything confusing as **Suggestion** or **Nitpick**

### 4. Navigability

The reader can find what they need without reading the whole document.

- Are there headings at the right level of granularity?
- Does a table of contents exist for long docs (> 200 lines)?
- Do cross-references (links to other docs, skills, agents) resolve?
- Flag broken links as **Blocking**, missing TOC as **Suggestion**

## Severity Levels

| Level      | Action required before publishing        |
| ---------- | ---------------------------------------- |
| Blocking   | Must fix — incorrect, broken, or missing |
| Suggestion | Should fix — incomplete or confusing     |
| Nitpick    | Optional — style, wording, minor clarity |

## Output Format

```
## Doc Review: <filename>

### Blocking
- <finding> (line N or section name)

### Suggestion
- <finding>

### Nitpick
- <finding>

**Verdict**: PASS (no blocking) | FAIL (blocking issues found)
```

A PASS means no blocking findings. Suggestions and nitpicks do not block publishing
but should be addressed before the next review cycle.

## Handoff

- Blocking findings → hand back to `godmode:doc-writer` or `godmode:doc-sync` to fix
- PASS → hand off to `godmode:cap` to commit the docs
