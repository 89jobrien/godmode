---
description: "Mixture-of-Agents architecture review pipeline. Analyze the current branch diff from five"
allowed-tools:
  - Bash
  - Read
  - Write
  - Edit
  - Glob
  - Grep
  - Agent
---

## Rules

- Read the full diff before commenting. Never review partial context.
- Group findings as Blocking / Suggestions / Nitpicks.
- Apply ALL severity levels in one pass before committing. Do not commit after
  fixing only blocking issues — one review, one fix commit.
- Run verification after fixes: `cargo clippy --workspace -- -D warnings`,
  `cargo nextest run --workspace`, `cargo fmt --all --check`.
- Never use `--no-verify` on git commits.
- Run `git branch --show-current` before any commit. If on main, STOP.

Mixture-of-Agents architecture review pipeline. Analyze the current branch diff from five
lenses in parallel, synthesize into a prioritized fix list, then dispatch TDD agents to
implement every actionable finding across all severity levels.

Arguments: $ARGUMENTS (optional: base branch, default: main)
Resolve empty input to `main`. Require one ref, reject whitespace or leading `-`, and validate
it with `git check-ref-format --branch "$base_branch"` before use.

## Phase 1: Parallel analysis (5 lenses)

Run `git diff "$base_branch...HEAD"` to get the full diff.
Write each analysis to `.ctx/godmode/review/` before synthesizing.

Dispatch 5 sub-agents in parallel, one per lens:

**Lens 1 — API surface & backward compatibility**
File: `.ctx/godmode/review/01-api-surface.md`

- List every changed public function, struct, trait, and type alias
- Flag any breaking changes (removed items, changed signatures)
- Flag any semver violations

**Lens 2 — Rust SOLID principles & module boundaries**
File: `.ctx/godmode/review/02-solid-modules.md`

- Single responsibility: do any modules/structs do too much?
- Dependency inversion: are concrete types leaking through ports?
- Module boundary violations: cross-crate coupling that should go through traits?

**Lens 3 — Test coverage gaps**
File: `.ctx/godmode/review/03-test-coverage.md`

- For every touched file, find its corresponding test file
- List public functions/methods with no test coverage
- Identify untested error paths and edge cases

**Lens 4 — Blast radius**
File: `.ctx/godmode/review/04-blast-radius.md`

- For every changed public function, list all callers across the workspace
- Flag callers that may be broken by signature or behaviour changes
- Run `cargo check --workspace` and capture any compilation errors

**Lens 5 — Documentation accuracy**
File: `.ctx/godmode/review/05-docs.md`

- Compare doc comments against implementation for all changed items
- Flag doc examples that won't compile or produce wrong output
- Flag missing `# Examples` on public items

## Phase 2: Synthesis

After all 5 agents complete, read all 5 `.ctx/godmode/review/` files.
Produce `.ctx/godmode/review/00-synthesis.md` with a prioritized table:

| Priority | Finding | Lens | File:Line | Fix |
| -------- | ------- | ---- | --------- | --- |
| HIGH     | ...     | ...  | ...       | ... |
| MED      | ...     | ...  | ...       | ... |
| LOW      | ...     | ...  | ...       | ... |

HIGH = breaking change, compilation error, security issue, missing test for error path
MED = SOLID violation, missing test for happy path, stale doc
LOW = nitpick, style, non-critical doc gap

Present the synthesis table to the user and ask for go/no-go before Phase 3.

## Phase 3: Parallel fix agents

Dispatch `godmode:gm-crate` fix agents for every actionable HIGH, MED, and LOW finding,
grouped into at most 5 non-overlapping worktree slots.
Each agent:

1. Verifies branch with `git branch --show-current` before any commit
2. Writes a failing test for the finding first
3. Implements the fix to make the test pass
4. Runs `cargo clippy --workspace -- -D warnings` — zero warnings
5. Derives a concrete `fix(scope): summary [moa-review]` commit message from its diff
6. Reports commit SHA and test name

Fix agents commit only to their assigned branches. The parent dispatcher owns every sequential
merge and all worktree cleanup. After all fix agents complete, merge sequentially and run the
full suite. Report findings resolved, any explicitly blocked findings, and total tests passing.
