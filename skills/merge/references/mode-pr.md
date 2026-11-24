# PR modes (create and merge)

Read this before opening a pull request for the current branch, or before merging an open PR via `gh`.

## Mode: PR-create

Open a pull request for the current branch.

```bash
git push -u origin $(git branch --show-current)

gh pr create \
  --title "<conventional commit title>" \
  --body-file - <<'EOF'
## Summary
- <bullet 1>
- <bullet 2>

## Test plan
- [ ] taskit test run passes
- [ ] clippy clean
EOF
```

Print the PR URL when done. Do not merge — leave that to the user or a subsequent `/merge` call.

## Mode: PR-merge

Merge an open PR via gh.

First, check the PR state:

```bash
gh pr view --json number,title,mergeable,mergeStateStatus,reviewDecision
```

If `mergeStateStatus` is not `CLEAN`, report the blockage and stop.

Choose merge style:

- Default: `--merge` (preserves merge commit, matches `--no-ff`)
- Squash requested: `--squash`
- Rebase requested: `--rebase` (only if no merge commits on branch)

```bash
gh pr merge <number> --merge --delete-branch
```

`--delete-branch` removes the remote branch after merge. Pair with worktree cleanup below if
this was a worktree branch.
