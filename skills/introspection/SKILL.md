---
name: "godmode:introspection"
description: >
  Audit godmode skills, agents, and plugin files for internal consistency, broken
  references, stale commands, and cross-skill contradictions. Use when asked to
  "introspect", "audit the skills", "review godmode", or after adding new skills.
---

# Introspection

Audit all skill and plugin files as a system — not just individual files.

## When to Use

- After adding or modifying skills
- Before a release/tag of the godmode plugin
- When a skill references a command that may not exist
- When asked to "introspect" or "audit godmode"

## Step 1: Inventory

Collect all skill files:

```bash
# List all SKILL.md files
# Glob: skills/**/SKILL.md
```

For each skill, note:

- Name (from frontmatter)
- Commands invoked (CLI calls, bash snippets)
- Tools used (Read, Write, Grep, Bash, etc.)
- Cross-references to other skills or helpers
- External commands assumed to exist

## Step 2: Check CLI correctness

Cross-reference every `godmode <subcommand>` call against the CLI reference in
`skills/using-godmode/SKILL.md` and `CLAUDE.md`.

Flag any subcommand that is not listed — it likely doesn't exist.

## Step 3: Check tool hygiene

Flag any skill that uses:

| Anti-pattern                  | Correct alternative                        |
| ----------------------------- | ------------------------------------------ |
| `cat <file>`                  | Use Read tool or note "use Read tool"      |
| `grep <pattern>`              | Use Grep tool or pipeline (`\| rg`)        |
| `find <dir>`                  | Use Glob tool                              |
| `git -p` / `add -p`           | `git add -A` or explicit pathspec          |
| `gh run watch`                | `gh run list --limit 3` (no TTY in agents) |
| Bare `op://` URIs in commands | `op read` or `op run`                      |
| `--no-verify` on commits      | Never                                      |
| `cd <dir> && git`             | `git -C <dir>`                             |

## Step 4: Check cross-skill consistency

Compare any instruction that appears in multiple skills. Flag contradictions:

- Merge strategy: must be `--no-ff merge` everywhere (never cherry-pick for parallel agents)
- Branch guard: every commit workflow must include `git branch --show-current` check
- Concurrency cap: must be `5` everywhere
- BLOCKED.md: trigger is always 3 failed attempts, never less

## Step 5: Check reference integrity

For every `**See also**`, `references/`, or `helpers/` link in a SKILL.md:

- Verify the referenced file exists (use Glob)
- Verify the section or heading exists if a fragment is cited

## Step 6: Check skill index completeness

Read `skills/using-godmode/references/skill-index.md` and `skills/using-godmode/SKILL.md`.

Every skill directory with a `SKILL.md` must have an entry in both. Flag any that are missing.

## Step 7: Report

Write a timestamped report to `.ctx/godmode/reports/introspection/introspection-<YYYY-MM-DD>.md`:

```markdown
# Introspect Report — <YYYY-MM-DD HH:MM>

## Blocking (breaks agent execution)

- [skill:line] <issue> — <why it matters>

## Suggestion (degrades reliability)

- [skill:line] <issue> — <recommendation>

## Nitpick (cosmetic or minor)

- [skill:line] <issue>

## No issues found

- <list items that were verified clean>
```

If no issues are found in any category, write the report anyway — record what was verified.

Use the Write tool to create `.ctx/godmode/reports/introspection/introspection-<YYYY-MM-DD>.md`. Overwrite if the file
already exists for today.

Also print a summary to stdout.

After writing the report, update the report index:

```bash
godmode report index-add --category introspection --file "introspection-<YYYY-MM-DD>.md"
```

If the CLI subcommand is not yet available, read
`.ctx/godmode/reports/godmode-reports.index.json`, add the filename to
`categories.introspection.files` (if not already present), and write it back
with the Write tool.

## Step 8: Fix

Apply all fixes in one pass. After fixing:

1. Re-run this skill's Steps 2–6 to confirm no new issues were introduced.
2. Update the report in `.ctx/godmode/reports/introspection/introspection-<YYYY-MM-DD>.md` with a `## Fixes Applied` section.
3. Commit with `fix(skills): introspect corrections — <summary>`.

## Guardrails

- Do not modify skills outside the flagged scope.
- Do not invent new CLI subcommands — only reference ones confirmed in CLAUDE.md or
  `using-godmode/SKILL.md`.
- A skill that references a nonexistent godmode subcommand is always Blocking severity.
