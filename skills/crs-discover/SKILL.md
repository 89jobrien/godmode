---
name: "godmode:crs-discover"
description: >
  Discover unhandled commands from Claude Code session history via crs discover.
  First step of the coursers-rules pipeline — writes rule candidates to
  .ctx/coursers/crs-candidates.json for crs-propose-rules to consume.
requires: []
next: [crs-propose-rules]
---

# crs-discover

Scan recent Claude Code session history for commands that no course-correct
rule currently handles, and persist them as rule candidates.

## When to Use

- First step of `godmode pipeline start coursers-rules`
- Asked to "find unhandled commands", "discover rule candidates", or
  "what commands keep slipping past the hooks"

## Steps

1. Ensure the output directory exists: `.ctx/coursers/`.
2. Run the discovery scan (last 30 days, only commands seen 3+ times):

   ```
   crs discover --format json --since 30 --min-count 3
   ```

3. Write the JSON output verbatim to `.ctx/coursers/crs-candidates.json`.
4. Report a one-line summary: number of candidate commands and the top 3 by
   occurrence count.

## Contract

- **Input**: Claude Code session history (crs reads it itself — no arguments
  beyond the flags above).
- **Output**: `.ctx/coursers/crs-candidates.json` — the raw `crs discover`
  JSON. Downstream skills depend on this exact path.
- Candidates below 3 occurrences are excluded by `--min-count 3`; do not
  lower this without being asked.
- If `crs` is not on PATH or discovery returns no candidates, write an empty
  JSON array to the output path and report that the pipeline has nothing to
  propose — the remaining steps then no-op cleanly.
