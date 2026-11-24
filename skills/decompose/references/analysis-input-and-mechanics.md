# Input identification and mechanical analysis

Read this before starting a decomposition: choosing the input diff, running `helpers/analyze-diff.nu`, or reading its coupling warnings.

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

---

## Step 1 — Mechanical analysis

Run the analyzer. This is deterministic — no LLM judgment yet.

```bash
nu skills/decompose/helpers/analyze-diff.nu \
  --base main \
  --branch $(git branch --show-current) \
  | save --force .ctx/godmode/decomps/$(git branch --show-current | str replace '/' '-')-analysis.json
```

The analyzer produces a JSON report with:

```json
{
  "files": [...],
  "crate_groups": { "crate-name": ["path/to/file.rs", ...] },
  "concern_groups": { "tests": [...], "deps": [...], "docs": [...], "logic": [...] },
  "coupling_warnings": [
    { "files": ["a.rs", "b.rs"], "reason": "shared import: crate::foo::Bar" }
  ],
  "proposed_splits": [
    { "id": 1, "files": [...], "crate": "crate-name", "concern": "logic", "coupled_to": [] }
  ]
}
```

Read and summarize the report. Surface all coupling warnings — these are files the mechanical
analysis grouped separately but that share a type or module import. They may need to move
together.
