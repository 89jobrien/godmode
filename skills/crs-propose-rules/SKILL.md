---
name: "godmode:crs-propose-rules"
description: >
  Draft course-correct rule stubs (pattern, message, exceptions) from the
  candidates crs-discover produced. LLM-assisted step of the coursers-rules
  pipeline — writes .ctx/coursers/crs-proposed-rules.json for crs-validate.
requires: [crs-discover]
next: [crs-validate]
---

# crs-propose-rules

Turn discovered unhandled commands into concrete course-correct rule
proposals. This is the judgment step: not every frequent command deserves a
blocking rule.

## When to Use

- After `crs-discover` has written `.ctx/coursers/crs-candidates.json`
- Asked to "propose rules for these commands" or "draft blocking rules"

## Steps

1. Read `.ctx/coursers/crs-candidates.json`. If empty, write an empty
   `rules` array to the output path and stop.
2. For each candidate, decide whether a rule is warranted. Propose a rule
   only when a dedicated tool or cheaper alternative exists (the same logic
   behind no-grep/no-cat: Grep tool over grep, Read tool over cat/sed -n).
   Skip candidates that are legitimate commands with no better alternative.
3. For each accepted candidate, draft a rule object matching the live
   config's schema (`~/.config/coursers/course-correct-rules.json`):

   ```json
   {
     "id": "no-<cmd>-use-<alternative>",
     "enabled": true,
     "pattern": "\\b<cmd>\\b",
     "pattern_flags": "",
     "target_commands": ["<cmd>"],
     "exceptions": [],
     "exception_policy": "allow_if_any_match",
     "message": "Use <alternative> instead. Example: <concrete example>."
   }
   ```

4. Author patterns defensively:
   - Anchor with `\b` word boundaries; never bare substrings.
   - Add exceptions for pipeline use (`\\|\\s*<cmd>`) and for `nu -c`
     wrapped commands (`nu\\s+-c\\b`) when the command is fine in those
     contexts — mirror the exception style of the existing no-grep rule.
   - Test each pattern before writing it out: `echo "<sample>" | crs probe`
     against a temp rules file, or at minimum verify the regex compiles.
5. Write `{"rules": [...]}` to `.ctx/coursers/crs-proposed-rules.json`.
6. Report each proposed rule id with a one-line rationale, and list the
   candidates you deliberately skipped with why.

## Contract

- **Input**: `.ctx/coursers/crs-candidates.json`
- **Output**: `.ctx/coursers/crs-proposed-rules.json` — object with a
  `rules` array in the live config schema. crs-validate merges this file;
  a malformed rule here fails the pipeline there, which is the intended
  safety net.
- Never edit `~/.config/coursers/course-correct-rules.json` in this step —
  installation is crs-install's job, after validation.
