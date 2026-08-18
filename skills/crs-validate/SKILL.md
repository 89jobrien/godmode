---
name: "godmode:crs-validate"
description: >
  Validate proposed course-correct rules merged with the live ruleset via
  crs validate. Gate step of the coursers-rules pipeline — a failure here
  stops the pipeline before crs-install can touch the live config.
requires: [crs-propose-rules]
next: [crs-install]
---

# crs-validate

Prove the proposed rules are healthy — patterns compile, examples fire,
exceptions work — against the merged (existing + proposed) ruleset before
anything is installed.

## When to Use

- After `crs-propose-rules` has written `.ctx/coursers/crs-proposed-rules.json`
- Asked to "validate these rules" before installing them

## Steps

1. Read the live config `~/.config/coursers/course-correct-rules.json` and
   the proposals `.ctx/coursers/crs-proposed-rules.json`. If the proposals
   file is missing or has an empty `rules` array, report "nothing to
   validate" and pass.
2. Reject any proposed rule whose `id` already exists in the live config —
   duplicates must be resolved in the proposal step, not silently merged.
3. Merge: live rules + proposed rules into a single config object, written
   to `.ctx/coursers/crs-merged-rules.json` (same top-level shape as the
   live file).
4. Validate the merged set:

   ```
   crs validate --rules .ctx/coursers/crs-merged-rules.json
   ```

5. Exit disposition:
   - `crs validate` exits 0 → step passes; leave the merged file in place
     for crs-install.
   - Non-zero exit or any reported error → **fail the step**. Report which
     rule(s) failed and why. Do NOT advance to crs-install; the pipeline
     must stop here (`godmode pipeline stop` if running interactively).

## Contract

- **Input**: `.ctx/coursers/crs-proposed-rules.json` + live config
- **Output**: pass/fail; on pass, `.ctx/coursers/crs-merged-rules.json`
- This step never writes to `~/.config/coursers/` — it is read-only with
  respect to live configuration.
