# CI

Read this before checking CI status for the current branch.

Inspect recent runs on the current branch without opening a TTY watcher:

```bash
gh run list --branch "$(git branch --show-current)" --limit 3
```
