---
name: gh-bulk-issues
description: Use when creating multiple GitHub issues from a structured
  list. Symptoms - need to file 3+ issues with consistent format/labels.
---

# Bulk GitHub Issue Creation

## When to Use

Filing 3+ related issues at once (e.g., refactor candidates, tech debt).

## Commands

Chain all `gh issue create` calls in a single Bash invocation using `&&`:

```bash
gh issue create -R OWNER/REPO --title "..." --label "LABEL" \
  --body "$(cat <<'EOF'
Body here (markdown).
EOF
)" && \
gh issue create -R OWNER/REPO --title "..." --label "LABEL" \
  --body "$(cat <<'EOF'
Body here.
EOF
)"
```

## Rules

- Always use HEREDOC for body (`$(cat <<'EOF' ... EOF)`)
- Check available labels first: `gh label list -R OWNER/REPO`
- Keep titles under 70 chars, prefix with type (`refactor:`, `feat:`, `fix:`)
- Batch all creates in one Bash call to avoid repeated user approvals

## Common Failures

| Symptom                | Fix                                                      |
| ---------------------- | -------------------------------------------------------- |
| `label not found`      | Run `gh label list` first; create with `gh label create` |
| Body formatting broken | Use `<<'EOF'` (single-quoted) to prevent shell expansion |
