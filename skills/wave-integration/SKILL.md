---
name: "godmode:wave-integration"
description:
  Use when merging multiple parallel agent branches (waves) into a single integration
  commit — rebasing each onto main, resolving conflicts while preserving intent, running tests
  per branch, and producing a clean summary commit with a conflict resolution log.
argument-hint: "[BRANCH_LIST]"
---

# Wave Integration Orchestrator

## Overview

Integrates parallel agent branches (a "wave") into main sequentially: rebase → resolve conflicts
→ test → repeat. Produces one integration commit summarizing all changes plus an explicit log of
every conflict resolved and the reasoning used.

## When to Use

- After dispatching parallel subagents (orca-strait, manual waves) and their branches are ready
- When you have `$ARGUMENTS` or `[BRANCH_LIST]` of feature branches all targeting the same base
- Before cutting a release that depends on multiple parallel streams of work

**Not for:** Single-branch merges, octopus merges, or branches that have diverged significantly
from each other (resolve those manually before invoking this skill).

## Helpers & References

| File                                        | Purpose                                                            |
| ------------------------------------------- | ------------------------------------------------------------------ |
| `helpers/wave-integrate.nu`                 | Automated rebase+test+merge loop; run directly or use as reference |
| `references/conflict-resolution-log.md`     | Filled example of a complete conflict log                          |
| `references/integration-commit-template.md` | Commit message templates for all integration outcomes              |

Run the helper:

```bash
wave-integrate --branches "feat/a feat/b feat/c" --base main
wave-integrate --branches "feat/a feat/b" --dry-run   # rebase+test only, no merge
```

---

## Steps

### 1. Inventory branches

```bash
git fetch --all
git branch -r | grep -E "<wave-prefix>"
```

For each branch confirm it compiles before touching it:

```bash
git checkout <branch> && cargo check --workspace 2>&1 | tail -3
```

### 2. Rebase each branch onto current main (in dependency order)

Process branches one at a time — never attempt an octopus merge.

```bash
git checkout main && git pull
git checkout <branch>
git rebase main
```

**If rebase conflicts:**

1. For each conflicted file, read BOTH sides:
   - `git show HEAD:<file>` — incoming (main) version
   - `git show REBASE_HEAD:<file>` — branch version
2. Identify the **intent** of each side — do not just pick one side mechanically.
3. Produce a merged version that preserves both intents.
4. Record the conflict in the resolution log (see Step 5).
5. `git add <file> && git rebase --continue`

**Never use `git rebase --skip`** — skipping drops commits and silently loses work.

### 3. Test after each rebase

```bash
cargo nextest run --workspace 2>&1 | tail -20
```

- If tests fail: debug and fix on the branch before proceeding to the next branch.
- If fix is non-trivial: stop and report to user rather than guessing.
- Cap at 3 fix attempts per branch before escalating.

### 4. Merge to main

Verify you are on main before merging (`guardrails.nu check-branch main`):

```bash
git branch --show-current   # must be main — stop if not
git checkout main
git merge --no-ff <branch> -m "integrate(<scope>): merge <branch>"
```

Use `--no-ff` to preserve branch topology in the log.

### 5. Build conflict resolution log

Maintain a running list during integration:

```
## Conflict Resolution Log

| File | Branch | Main side intent | Branch side intent | Resolution |
|------|--------|-----------------|-------------------|------------|
| crates/foo/src/lib.rs | feat/x | Added new error variant | Renamed existing variant | Kept rename, added new variant after |
```

One row per file per conflict. Be specific about intent, not just "kept both".

### 6. Final integration commit

After all branches are merged and `cargo test --workspace` passes:

```bash
git commit --allow-empty -m "chore(integration): wave N integration

Branches merged:
- feat/branch-a (sha)
- feat/branch-b (sha)

Conflicts resolved: N files (see below)
<paste resolution log>"
```

If there were zero conflicts, note that explicitly.

### 7. Report to user

Summarize:

- Branches integrated (with final SHAs on main)
- Test result
- Conflict resolution log (full table)
- Any branches skipped or escalated, and why

## Conflict Resolution Heuristics

| Scenario                                          | Resolution                                     |
| ------------------------------------------------- | ---------------------------------------------- |
| Both sides add to a list/enum                     | Keep all additions                             |
| One side renames, other adds using old name       | Apply rename, update the addition              |
| Both sides modify same function differently       | Compose if orthogonal; ask user if overlapping |
| One side deletes, other modifies                  | Prefer the modification; note deletion intent  |
| Cargo.toml dependency version conflict            | Take the higher version                        |
| Both sides add the same dep at different versions | Take the higher version                        |
| Test file conflicts                               | Keep all tests; dedup if identical             |

## Common Mistakes

- **Merging all branches at once (octopus)** — never. Sequential rebase only.
- **Using `--skip` to clear conflicts** — drops commits silently; always resolve.
- **Not testing between branches** — one broken branch can hide failures in the next.
- **Vague conflict log ("kept both")** — always state the _intent_ of each side.
- **Committing to main directly** — always rebase the branch, then merge to main.
