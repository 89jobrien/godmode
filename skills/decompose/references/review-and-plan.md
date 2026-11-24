# LLM review of splits and the decomposition plan

Read this before judging the mechanical groupings, naming splits, or presenting the plan for approval. Do not run any git mutation until the user has approved the plan.

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

```text
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
