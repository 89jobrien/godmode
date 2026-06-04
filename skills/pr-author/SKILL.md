---
name: "godmode:pr-author"
description: >
  Compose PR descriptions from branch diffs, task context, and commit history.
  Produces structured descriptions ready for `gh pr create`. Uses conventional
  commit style and links godmode tasks.
requires: []
next: [merge]
---

# PR Author

Write structured PR descriptions directly from branch diffs and task context.

## When to Use

- After completing a feature or fix on a branch
- Before running `gh pr create`
- When you need a consistent PR structure with summary, changes, test plan, and
  linked issues

## PR Structure

### Title

Format: `<type>(<scope>): <description>`

- **Type**: `feat`, `fix`, `refactor`, `test`, `docs`, `chore`
- **Scope** (optional): crate or component name
- **Description**: concise one-liner, under 70 characters total
- **No period at the end**

Examples:

- `feat(godmode-core): add task dependency validation`
- `fix: correct task status persistence on reload`
- `refactor: simplify session trait bounds`

### Summary

2-3 sentences describing what changed and why. Lead with the user-facing impact
or the problem solved:

- "Adds support for..."
- "Fixes incorrect behavior..."
- "Refactors... to improve..."

### Changes

Grouped by crate, with bullet points per area:

```
### Crate: godmode-core
- model: add `depends_on` field to Task struct
- graph: implement dependency resolution in `runnable()`
- session: track blocked task metadata

### Crate: godmode-cli
- cli: add `--depends-on` flag to `task add`
```

### Test Plan

Describe what was tested:

- "All existing tests continue to pass"
- "Ran `cargo nextest run --workspace`"
- "Added new tests in `tests/unit/` for dependency resolution"
- "Verified behavior with manual test: <steps>"

Always include at least one testing bullet point.

### Linked Issues

If the branch addresses GitHub issues or godmode tasks:

```
Closes #42 (fix: task list not updating on status change)
Relates to #38 (design: task dependency syntax)
Completes godmode tasks: t1, t3, t5
```

Include the issue title or task description if helpful.

## Task Graph Integration

The PR author reads `.ctx/godmode/tasks.yaml` to identify which tasks are marked
`done` on this branch. These are listed in the "Linked Issues" section as
godmode task IDs (e.g., `t1`, `t3`).

## Commit History

The PR author reads `git log main..HEAD --oneline` to:

1. Count commits by conventional-commit type
2. Identify scopes (crate names) from commit messages
3. Search for linked issue numbers in commit bodies via `--format=%B`

## gh pr create Integration

After the user approves the draft, the agent executes:

```bash
gh pr create --title "<title>" --body "<body>"
```

The title and body are shown in the draft step, so the user can verify before
creation.

The agent never creates a PR without explicit user approval of the draft.

## Conventions

- All titles follow conventional-commit format (required for changelog
  integration)
- All changes are grouped by crate for readability
- All PRs include a test plan
- All linked issues are verified by reading git log or task YAML

## See also

- `gm-code-review-agent` — run a code review BEFORE running this agent
- `gm-changelog-agent` — generates changelog from commit history
- GitHub CLI: `gh pr create --help`
