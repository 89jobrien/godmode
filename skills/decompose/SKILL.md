---
name: "godmode:decompose"
description: >
  Use when the user wants to break up a large diff, PR, or branch into smaller independent PRs
  or branches. Handles mechanical analysis (file-to-crate mapping, concern classification, coupling
  detection), proposes a decomposition plan, requires explicit approval, then executes the split
  (creates branches, stages subsets, runs cargo check per split, optionally opens PRs). Trigger
  on "decompose this PR", "split this branch", "this diff is too big", "break this into smaller
  PRs", "make this reviewable", "/decompose", or any request to split a large changeset into
  independent units.
requires: []
next: []
allowed-tools: Bash, Read, Glob, Grep
---

# godmode:decompose — Diff Decomposition Agent

Splits a large diff or branch into independent, reviewable units. Mechanical analysis runs first
and is fully verifiable. LLM judgment is used only where the mechanical analysis is ambiguous —
and always requires explicit user approval before any git mutations occur.

## Helpers

| File                             | Purpose                                                                         |
| -------------------------------- | ------------------------------------------------------------------------------- |
| `helpers/analyze-diff.nu`        | Mechanical analysis: file→crate map, concern classification, coupling detection |
| `helpers/split-branch.nu`        | Branch creation, file staging, per-split verification loop                      |
| `helpers/decompose-proptest.rs`  | Property tests for grouping and coverage invariants                             |
| `references/split-strategies.md` | Decision guide for ambiguous grouping calls                                     |

---

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

---

## Step 2 — LLM review of proposed splits

Review the mechanical groupings with judgment:

1. **Check coupling warnings** — if two proposed splits share a public API type or trait impl,
   merge them. The mechanical analysis detects imports but not semantic dependency. Read
   `references/split-strategies.md` for decision heuristics.

2. **Check group sizes** — a split with 1 file and a split with 40 files suggests the coarse
   grouping needs subdivision. Apply concern sub-classification (see split strategies).

3. **Name each split** — use a conventional branch name:
   `<source-branch>-split-<N>-<scope>` (e.g., `feat/auth-split-1-core`, `feat/auth-split-2-tests`)

4. **Write a one-line rationale** for each split: why these files belong together and why they
   are independent from the other splits.

---

## Step 3 — Present the decomposition plan

Show the full plan before doing anything. Format:

```
Decomposition plan for: <branch> (N files changed)

Split 1: <name>
  Branch: <branch-name>
  Files: N files
    - crates/foo/src/lib.rs
    - crates/foo/src/bar.rs
  Rationale: <one line>
  Coupling risk: none | <warning if any>

Split 2: <name>
  ...

Coupling warnings (review before approving):
  - a.rs and b.rs both import crate::foo::Bar — confirm they can ship independently

Dry run? [y/n] — reply 'go' to execute, 'adjust' to modify the plan, or give me new groupings.
```

**Do not proceed until the user explicitly confirms.** This is the guardrail. Every split that
is created after this point was approved by the user.

---

## Step 4 — Execute the splits

Run the splitter for each approved split:

```bash
nu skills/decompose/helpers/split-branch.nu \
  --source <source-branch> \
  --base main \
  --split-id 1 \
  --branch <split-branch-name> \
  --files "path/a.rs path/b.rs ..." \
  --output-dir .ctx/godmode/decomps/<decomp-name>
```

The splitter:

1. Checks out `main` (or base)
2. Creates the split branch
3. Cherry-picks by file: `git checkout <source-branch> -- <files>`
4. Runs `cargo check --workspace`
5. Runs `cargo nextest run --workspace` (or scoped to affected crates)
6. If checks pass: commits with a conventional message
7. If checks fail: reports failure, rolls back the branch, stops — does not continue to next split

Process splits sequentially, not in parallel. A failing split must be resolved before continuing.

After all splits pass:

```bash
# Verify no file was dropped — writes result to .ctx/godmode/decomps/
nu skills/decompose/helpers/analyze-diff.nu --verify-coverage \
  --source <source-branch> \
  --splits "split-1-branch split-2-branch ..." \
  | save --force .ctx/godmode/decomps/<decomp-name>/coverage.json
```

Coverage check: the union of all split files must equal the source branch file list. Report any
files that appear in the source but no split (orphaned files).

---

## Step 5 — Open PRs (optional)

If the user wants PRs opened:

```bash
gh pr create \
  --base main \
  --head <split-branch> \
  --title "<conventional title for this split>" \
  --body "$(cat <<'EOF'
## Summary
<rationale from plan>

## Part of
Decomposed from #<original-PR-number-or-branch>.
Depends on: #<prior-split-PR> (if sequential dependency exists)

## Files
- crates/foo/src/lib.rs
- crates/foo/src/bar.rs

## Test plan
- [ ] cargo nextest run --workspace passes
- [ ] clippy clean
EOF
)"
```

Print each PR URL as it's created.

---

## Step 6 — Report

```
Decomposition complete: <source-branch> → N splits

Split 1: <name> | <sha> | PR #<N> <url>
Split 2: <name> | <sha> | PR #<N> <url>
...

Coverage: all N changed files accounted for.
Orphaned: none | <list if any>

Source branch <source-branch> was not modified.
```

If any split failed verification, list it as `FAILED` with the failure summary. The source branch
is always intact regardless.

---

## Guardrails

- **Source branch is read-only.** Never commit to it, never delete it. All splits branch from base.
- **Plan approval is required.** No git mutations before the user says go.
- **Sequential verification.** Each split must pass `cargo check` + `nextest` before the next
  begins. A failing split halts the process — do not skip forward.
- **Coverage check.** After all splits, verify every changed file appears in exactly one split.
  Files in multiple splits or no split are bugs.
- **Coupling warnings are not blockers** — but they must be shown. The user decides whether to
  merge the affected splits or accept the dependency.
- **Dry-run by default** on ambiguous inputs. If uncertain what the user wants decomposed, ask
  rather than acting.
