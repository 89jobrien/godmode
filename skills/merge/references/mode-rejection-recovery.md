# Rejection-recovery mode

Read this before recovering from a rejected `git push`, or when the push output mentions `non-fast-forward`, `fetch first`, `stale info`, or `Permission denied`.

## Mode: Rejection-recovery

```bash
git push 2>&1
```

| Output contains                    | Cause                             | Action                           |
| ---------------------------------- | --------------------------------- | -------------------------------- |
| `non-fast-forward` / `fetch first` | Remote has commits you don't have | Fetch, integrate, retry          |
| `rejected ... (stale info)`        | Force-push needed                 | Ask user — feature branches only |
| `Permission denied`                | Auth failure                      | Check SSH key / 1Password agent  |

```bash
git fetch origin
git log --oneline HEAD..origin/<branch>   # commits on remote you don't have
git log --oneline origin/<branch>..HEAD   # your commits not yet on remote
```

If only remote has new commits: continue to Direct-merge mode.
If both sides diverged: resolve conflicts, then push.
Never force-push to `main`.
