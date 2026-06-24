---
name: "godmode:changelog"
description: >
  Parse git history into structured changelogs. Use to generate release notes,
  audit commit conventions before tagging, or integrate with `godmode release`
  workflow.
requires: []
next: [release-notes]
---

# Changelog

## When to Use

- Before a release — to audit commits and generate release notes
- To understand what changed since the last tag
- To flag commits that do not follow conventional-commit format
- To group commits by type and affected crate
- To integrate with `godmode release changelog` subcommand

## Conventional Commit Format

Commits should follow the pattern:

```
<type>(<scope>): <description>
```

- **Type** (required): `feat`, `fix`, `refactor`, `chore`, `docs`, `test`, `ci`
- **Scope** (optional): the affected crate or module, e.g. `godmode-core`,
  `godmode-cli`
- **Description** (required): concise summary of the change

Examples:

```
feat(godmode-core): add changelog generator
fix(godmode-cli): handle missing config file
refactor: unify error handling in dispatch module
docs: update CLI reference
test: add property tests for graph cycles
```

## Parsing Rules

1. Extract type from the commit message prefix (word before `(` or `:`)
2. If the type is not in the canonical list, mark as non-conventional
3. Extract scope from parentheses, if present (e.g. `godmode-core`)
4. Extract description from the text after `: ` or after the type
5. Group commits by type, then by scope/crate

## Output Format

Produce grouped markdown changelog:

```markdown
# Changelog — <tag>..HEAD

## Features

### <crate>

- <commit description>
- <commit description>

## Fixes

### <crate>

- <commit description>

## Refactoring

...

## Other

...

## Non-Conventional Commits

- <full commit message> (reason: missing type, scope, or separator)
```

Omit sections with no entries. "Other" includes `docs`, `test`, `chore`, `ci`
and commits without a crate scope.

## Integration with Release Workflow

Write the generated changelog to `CHANGELOG.md` or `.ctx/` manually and commit.
(`godmode release changelog` is not implemented in the CLI — generate output from
this skill and write it with the Write tool.)

## Guardrails

- Never fabricate commits — read only from `git log`
- Never rewrite git history
- Never commit changes without explicit instruction
- Flag all non-conventional commits; do not silently omit them
- If asked to reword commits, refuse — use `git rebase` outside godmode
