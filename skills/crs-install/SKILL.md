---
name: "godmode:crs-install"
description: >
  Install validated course-correct rules into the live coursers config.
  Final step of the coursers-rules pipeline — backs up the config, swaps in
  the merged ruleset atomically, and re-validates the live file.
requires: [crs-validate]
next: []
---

# crs-install

Promote the validated merged ruleset to the live config at
`~/.config/coursers/course-correct-rules.json`.

## When to Use

- Final step of `godmode pipeline start coursers-rules`, only after
  crs-validate passed
- Asked to "install the validated rules"

## Steps

1. Preconditions — refuse to proceed (fail the step) if any is unmet:
   - `.ctx/coursers/crs-merged-rules.json` exists (crs-validate's output)
   - `crs validate --rules .ctx/coursers/crs-merged-rules.json` exits 0
     (re-run it; the file may have been edited since validation)
2. Back up the live config next to itself:
   `~/.config/coursers/course-correct-rules.json.bak-<YYYYMMDD-HHMMSS>`
3. Install atomically: copy the merged file to a temp name **in
   `~/.config/coursers/`** (same filesystem), then `mv` it over
   `course-correct-rules.json`. Never write the live file in place.
4. Confirm health of the installed config:

   ```
   crs validate
   ```

   Non-zero exit → restore the backup immediately, then fail the step with
   the validator output.

5. Smoke-test one newly installed rule end-to-end through the real hook
   path: pipe a matching payload to `crs hook pre-tool-use` and confirm a
   deny response.
6. Report: rules installed (ids), backup path, and the smoke-test result.
7. Clean up `.ctx/coursers/crs-merged-rules.json` and
   `crs-proposed-rules.json` (candidates file may stay for audit).

## Contract

- **Input**: `.ctx/coursers/crs-merged-rules.json` (validated)
- **Output**: updated `~/.config/coursers/course-correct-rules.json` + a
  timestamped backup
- This step modifies live hook configuration for every Claude session —
  the backup and post-install `crs validate` are mandatory, not optional.
