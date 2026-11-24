# Worktree cleanup and task state sync

Read this before removing a worktree after a merge, or before marking a godmode task done for a merged branch.

## Worktree cleanup

After a successful merge, check if the source branch was a worktree:

```bash
git worktree list
```

If the merged branch appears as a worktree path, remove it:

```bash
git worktree remove <path> --force
git branch -d <branch>
```

Only do this after confirming the branch is fully merged:

```bash
git log --oneline main..<branch>   # must be empty
```

## Task state sync

If a godmode task is associated with this branch, mark it done:

```bash
godmode task done <task-id> --commit $(git rev-parse HEAD) --notes "merged <branch> into main"
```

If no task ID is known, check `.ctx/godmode/tasks.yaml` for tasks in `running` state that match the
branch name or scope.
