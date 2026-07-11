---
name: "godmode:doc-writer"
description: >
  Write new documentation grounded in actual code. Use when a README, architecture doc,
  API reference, or CLAUDE.md does not exist or needs to be written from scratch.
  Triggers on "write docs", "document this", "add a README", "write an architecture doc".
requires: []
next: [doc-review]
---

# Doc Writer

Write documentation that reflects what the code actually does. Never invent features,
flags, or behaviour — read the source first.

## When to Use

- No README exists for a crate, skill, or directory
- CLAUDE.md is missing or empty
- An API or module needs a reference doc written from scratch
- Architecture or design decisions need to be captured after the fact

## Process

### Step 1: Read before writing

Before drafting a single line, read:

- All `Cargo.toml` files in scope (name, description, version, deps)
- Entry points: `src/lib.rs`, `src/main.rs`, `src/bin/*.rs`
- Public API surface: all `pub fn`, `pub struct`, `pub enum`, `pub trait`
- CLI help: `<binary> --help` and `<binary> <subcommand> --help`
- Existing docs if any (even stale ones reveal intent)
- `git log --oneline -20` for recent context

### Step 2: Identify the doc type

| Type             | Contents                                                             |
| ---------------- | -------------------------------------------------------------------- |
| README           | What it is, install, quickstart, key concepts, links to deeper docs  |
| CLAUDE.md        | Build commands, architecture, conventions, agent instructions        |
| Architecture doc | Component ownership, data flow, key decisions, tradeoffs             |
| API reference    | Every public export: signature, purpose, parameters, return, example |
| Skill/agent doc  | When to use, process steps, output format, next skill                |

### Step 3: Write in sections, verify each

Write one section at a time. After each section:

- Re-read the source to confirm every claim is accurate
- Flag any claim you cannot verify as `[UNVERIFIED]` rather than guessing

### Step 4: Cross-check

Before finishing:

- Every CLI flag documented → confirm it exists in `--help`
- Every file path mentioned → confirm it exists
- Every module/crate name → confirm it matches `Cargo.toml`
- Remove `[UNVERIFIED]` markers or resolve them

## Output Standards

- Code blocks for all commands and paths
- Tables for flag/option references
- No placeholder text — every section must be complete or explicitly marked TODO
- 100-column line width for markdown prose
- No invented examples — use real command output or real code snippets

## Handoff

After writing, run `godmode:doc-review` to catch accuracy and completeness issues
before committing.
