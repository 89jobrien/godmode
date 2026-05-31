# TODO Coverage Rules

A TODO is **covered** if any of these conditions are met:

1. The TODO comment itself contains an issue reference:
   - `// TODO(#42): ...`
   - `// TODO: JOB-123 — ...`
   - `// FIXME(gh-7): ...`

2. An open GitHub issue mentions the file path or TODO text in its title or body.

3. An open Linear issue matches the file or description.

## Not Covered

A TODO is **uncovered** if none of the above conditions are met. Uncovered TODOs
represent untracked work.

## Skip Rules

Do not create issues for TODOs in:

- `target/` (build output)
- `.git/` (version control internals)
- `node_modules/` or `vendor/` (third-party code)
- Generated files (check for `// Code generated` headers)
- Files marked for deletion in the current wave/plan

## Rate Limit

Cap issue creation at 20 per run. If more than 20 uncovered TODOs exist,
prompt the user for confirmation and prioritize by:

1. TODOs in actively-maintained code (recent commits to the file)
2. TODOs with urgency markers (FIXME > HACK > TODO > XXX)
3. TODOs in core crates over peripheral ones
