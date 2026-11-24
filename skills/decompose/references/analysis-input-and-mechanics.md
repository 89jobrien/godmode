# Input identification

Read this before starting a decomposition, to choose the input diff and record the source branch, base, and changed files. The mechanical analysis is in analysis-mechanics-step-1.md.

## Step 0 — Identify the input

Determine what to decompose:

```bash
# Option A: current branch vs main
git diff main...HEAD --name-only

# Option B: a specific PR
gh pr diff <number> --name-only

# Option C: patch file provided by user — read it directly
```

If the branch has more than one commit, also inspect the commit log for structure clues:

```bash
git log --oneline main..HEAD
```

Record:

- **Source branch**: the branch being decomposed (never modified)
- **Base**: the branch to split against (default: `main`)
- **Changed files**: full list with paths
