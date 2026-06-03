---
name: "godmode:cross-issue"
description: >
  Cross-repo issue coordination. Use when a feature spans multiple repos and
  needs coordinated issue tracking. Creates linked GitHub issues with
  cross-references and optionally creates Linear issues for planning.
---

# Cross-Repo Issue Linker

Link issues across multiple repositories when a feature or change spans
multiple projects. Maintains bidirectional cross-references and produces a
summary table for tracking.

## When to Use

- Multi-repo features (e.g., crux DSL change + godmode skill update)
- Breaking changes affecting multiple projects
- Coordinated releases or refactors across the workspace
- Long-running initiatives requiring planning across teams

## Issue Creation Workflow

### Phase 1: Planning

1. **Identify affected repos**: List all repos that need work
2. **Draft issue per repo**: Title, body, acceptance criteria
3. **Show the plan**: Present as a table; wait for approval
4. **Clarify dependencies**: Does repo A block repo B?

### Phase 2: Create and Link

1. **Create each issue** via `gh issue create --repo <owner/repo>`
2. **Capture issue numbers** from the created issues
3. **Add cross-references** to each issue body linking to the others
4. **Verify links** by viewing each issue and confirming references are visible

### Phase 3: Track

1. **Update status** as issues move through their workflows
2. **Check blocked issues** — if one repo is blocked, escalate
3. **Close issues** only when work is committed and merged

## Cross-Reference Format

Use this format in every issue body:

```markdown
## Related Issues

- [repo-label] owner/repo#N — brief description
- [other] owner/other#M — description
```

Example:

```markdown
## Related Issues

- [crux] owner/crux#42 — DSL grammar changes
- [devloop] owner/devloop#88 — Runtime adapter for new DSL type
```

For breaking changes, prefix with `[breaking]`:

```markdown
- [breaking][crux] owner/crux#42 — Breaking DSL change
```

## Linear Integration (Optional)

Create an umbrella Linear issue for planning across all repos:

1. **Fetch Linear API key** from 1Password (see CLAUDE.md for vault/item details)
2. **Create Linear issue** with project links to all affected repos
3. **Link all GitHub issues** in the Linear description

Linear issues are useful for:

- Tracking planning and design work before implementation starts
- Grouping independent GitHub issues under one initiative
- Sharing context with non-GitHub users (designers, PMs)
- Estimating scope across repos

Do not create a Linear issue if the scope is small or already tracked.

## Status Tracking

After creation, maintain a table in your session notes:

| Repo          | Issue | Status | Blocker             | Notes |
| ------------- | ----- | ------ | ------------------- | ----- |
| owner/crux    | #42   | Open   |                     |       |
| owner/godmode | #123  | Open   |                     |       |
| owner/devloop | #88   | Draft  | Waiting for crux#42 |       |

Update as work progresses. Mark issues `Draft` (not yet created), `Open`
(ready for work), `In Progress`, `Blocked`, or `Done`.

## Queries

Check for existing issues before creating duplicates:

```bash
# Find issues by title
gh issue list --repo owner/repo --search "title:feature-name"

# Find issues with a specific label
gh issue list --repo owner/repo --label cross-repo

# Find issues assigned to you
gh issue list --repo owner/repo --assignee @me
```

## Anti-Patterns

- Creating separate Linear and GitHub issues for the same work (pick one)
- Forgetting to add cross-references after issue creation (breaks navigation)
- Linking issues after closing them (no visibility in issue tracker)
- Creating issues without acceptance criteria (unclear when work is done)
- Not showing the plan before creating issues (surprises team members)

## Guardrails

- Every issue must reference all related issues in its body.
- Never close an issue without commits or explicit resolution.
- If an issue already exists, link to it — do not create duplicates.
- Use `[breaking]` prefix for breaking changes across repos.
- Approval required (Step 3 in agent workflow) before creating issues.
- Cross-references must be bidirectional (A links to B, B links to A).
