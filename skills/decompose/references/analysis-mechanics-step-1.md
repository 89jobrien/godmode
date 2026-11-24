# Mechanical analysis: running the analyzer

Read this when running helpers/analyze-diff.nu on the input diff, or when reading its coupling warnings before grouping the files.

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
