# Opening PRs for the splits

Read this only when the user has asked for PRs to be opened for the split branches. Do not open PRs by default.

## Step 5 — Open PRs (optional)

If the user wants PRs opened:

```bash
gh pr create \
  --base main \
  --head <split-branch> \
  --title "<conventional title for this split>" \
  --body-file - <<'EOF'
## Summary
<rationale from plan>

## Part of
Decomposed from #<original-PR-number-or-branch>.
Depends on: #<prior-split-PR> (if sequential dependency exists)

## Files
- crates/foo/src/lib.rs
- crates/foo/src/bar.rs

## Test plan
- [ ] taskit test run passes
- [ ] clippy clean
EOF
```

Print each PR URL as it's created.
