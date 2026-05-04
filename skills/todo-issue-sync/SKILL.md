---
name: "godmode:todo-issue-sync"
description: >
  Ensure every inline TODO comment in the codebase has a corresponding GitHub or Linear issue.
  Use when asked to "sync TODOs to issues", "make sure TODOs have issues", or "audit TODOs".
  Scans for TODO markers, cross-references open issues, and creates missing ones.
---

# TODO → Issue Sync

Scan the codebase for inline `TODO` markers and ensure each has a tracked issue in GitHub
or Linear.

## Step 1: Collect TODOs

Use the Grep tool to search the working tree for `TODO`, `FIXME`, `HACK`, and `XXX`
markers. Limit the search to source file types such as `*.rs`, `*.go`, `*.nu`, `*.ts`,
and `*.py`, and exclude generated or vendored paths such as `target/`, `.git/`,
`node_modules/`, `vendor/`, and generated files.

Deduplicate by file+line. Build a table:

| File | Line | TODO text |
| ---- | ---- | --------- |

Count them. Never silently truncate — capture ALL.

## Step 2: Check for existing issues

### GitHub (default)

```bash
gh issue list --state open --limit 100 --json number,title,body
```

Search titles and bodies for references to each TODO's file path or description.
A TODO is **covered** if an open issue mentions the file path or the TODO text.

### Linear (if the project uses Linear)

Use the `mcp__claude_ai_Linear__list_issues` tool with a query matching the file name
or TODO description. A TODO is covered if a matching non-completed issue exists.

### Inline issue reference

A TODO is also covered if the comment itself contains an issue number, e.g.:
`// TODO(#42): fix this` or `// TODO: JOB-123 — implement`.

## Step 3: Report gaps

Present a table of uncovered TODOs:

```
Uncovered TODOs (N):
  src/foo/bar.rs:42   TODO: implement rate limiting
  crates/x/src/lib.rs:7  TODO: add error handling
```

Ask for confirmation before creating issues: "Create issues for all N uncovered TODOs?"

## Step 4: On-demand task graph sync

Before creating new issues, sync the godmode task graph from GitHub to pick up any issues
that were created outside this session:

```bash
godmode task pull --github --repo <owner>/<repo>
```

This updates `.ctx/GODMODE.tasks.yaml` with any open GitHub issues not yet in the graph.
Re-check coverage after the pull — a previously uncovered TODO may now be tracked.

## Step 5: Create missing issues

For each uncovered TODO, create an issue with:

- **Title**: `fix/feat(<module>): <todo text>` — infer fix vs feat from context
- **Body**: includes file path, line number, full TODO text, and surrounding context (±5 lines)
- **Labels**: infer from directory (e.g. `crates/langchainx-tools/` → `tools`)
- **Priority**: Normal (3) by default

### GitHub

```bash
gh issue create \
  --title "feat(<module>): <todo text>" \
  --body "## Context\n\n\`<file>:<line>\` has an unresolved TODO.\n\n## TODO\n\n\`\`\`\n<todo text>\n\`\`\`\n\n## Action\n\nResolve or implement the TODO at \`<file>:<line>\`." \
  --label "<label>"
```

Capture the issue number from the URL printed by `gh issue create` (last path segment).

After each issue is created, add it to the godmode task graph immediately:

```bash
godmode task add gh-<N> "<title>"
```

Use `gh-<N>` as the task ID (e.g. `gh-42`) so it is traceable back to the GitHub issue.

### Linear

Use `mcp__claude_ai_Linear__save_issue` with team inferred from repo/project context.
After creating, add to the task graph:

```bash
godmode task add <linear-id> "<title>"
```

Use the Linear issue ID (e.g. `JOB-268`) as the task ID.

## Step 6: Annotate (optional)

If the user confirms, annotate each TODO comment with the new issue number:

```
// TODO(#<N>): implement rate limiting
```

Use the Edit tool for each file. Do not annotate if the user declines.

## Step 7: Summary

Report:

- Total TODOs found
- Already covered (with issue numbers)
- Newly created issues (with URLs)
- Any TODOs skipped (e.g. in legacy/deprecated paths)

## Guardrails

- Never create duplicate issues — search before creating.
- Never modify TODO text, only append the issue reference in parentheses.
- Skip TODOs in `target/`, `.git/`, `node_modules/`, `vendor/`, and generated files.
- If a TODO is in a file marked for deletion (Wave N cleanup), note it but do not create an issue.
- Cap issue creation at 20 per run — prompt for confirmation if more.
