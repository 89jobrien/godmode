---
name: "godmode:todo-issue-sync"
description: >
  Ensure every inline TODO comment has a corresponding GitHub issue using the native,
  resumable godmode synchronizer.
requires: []
next: []
---

# TODO → Issue Sync

Use the native command instead of invoking `gh issue list` or `gh issue create` directly.
It scans source comments for `TODO`, `FIXME`, `HACK`, and `XXX`, excluding generated,
vendored, build, worktree, and state directories.

## Preview

```bash
godmode issue sync-todos --preview [--repo owner/repo]
```

Preview is the default. It reads all open GitHub issue pages, reports covered and missing
markers, and performs no issue creation or state writes. Add `--json` for structured output.
Review every missing marker before applying.

Each marker receives a stable fingerprint derived from its path and normalized text rather
than its line number. Existing issues are matched by the hidden fingerprint in their body;
inline references such as `TODO(#42)` are already covered. If duplicate issues contain the
same fingerprint, the lowest issue number wins deterministically.

## Apply

```bash
godmode issue sync-todos --apply [--repo owner/repo]
```

Apply creates only missing issues. Each issue body records the source location, TODO text,
and fingerprint. Progress is atomically persisted after every creation in
`.ctx/godmode/todo-issue-sync.json`, so rerunning the same command resumes after interruption.
The synchronizer also refreshes all issue pages before creation, preventing duplicates when
an issue was created but local progress was not saved.

## Summary

Report total, covered, missing, and created counts plus any command error. On failure, fix the
GitHub CLI authentication or network issue and rerun the same apply command.

## Guardrails

- Preview before apply.
- Never manually create issues for entries reported as covered.
- Do not edit or annotate TODO comments during synchronization.
- Use `--repo owner/repo` when the current directory does not identify the intended repository.
