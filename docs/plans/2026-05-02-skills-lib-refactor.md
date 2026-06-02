# Plan: Skills `_lib/` Refactoring

**Status**: done

## Goal

Extract duplicated prose and command sequences from skill SKILL.md files into a set of
Nushell function libraries under `skills/_lib/`. Each library exports both runnable
validators (for use in helper scripts) and template generators (for use in agent prompt
construction). Skills reference the libraries instead of copy-pasting rules.

## Architecture

**Crates affected:** none — this is skills-only, no Rust changes.

**New files:**

| File                          | Role                                                                    |
| ----------------------------- | ----------------------------------------------------------------------- |
| `skills/_lib/quality-gate.nu` | nextest + clippy + fmt runner and command-block generator               |
| `skills/_lib/guardrails.nu`   | branch guard runner, branch-guard prose generator, blocked-rule prose   |
| `skills/_lib/review-rules.nu` | severity table, false-positive protocol, one-pass rule prose generators |
| `skills/_lib/dispatch.nu`     | worktree setup, agent prompt builder, wave-state init, integration loop |

**Skills modified (prose only — no behaviour change):**

| Skill                            | What changes                                                   |
| -------------------------------- | -------------------------------------------------------------- |
| `cap`                            | Gate steps → `quality-gate.nu`; branch guard → `guardrails.nu` |
| `ci-fix`                         | Branch guard → `guardrails.nu`                                 |
| `test-driven-development`        | Gate steps → `quality-gate.nu`; branch guard → `guardrails.nu` |
| `refactoring`                    | Gate steps → `quality-gate.nu`                                 |
| `verification-before-completion` | Gate steps → `quality-gate.nu`                                 |
| `code-review`                    | Severity table + false-positive protocol → `review-rules.nu`   |
| `receiving-review`               | Same                                                           |
| `parallel-agents`                | Dispatch section → `dispatch.nu`                               |
| `tackle-issues`                  | Worktree + agent + integrate sections → `dispatch.nu`          |

**Skills unchanged:**
brainstorm, systematic-debugging, writing-plans, task-management, testing-philosophy,
observability-as-infrastructure, introspection, moa, wave-integration, using-godmode,
wave-integration.

## Tech Decisions

- **Nushell only** — consistent with `_lib/trace.nu` precedent; no bash.
- **Dual export per concern** — every library exports both a runner (`def run-*`) and a
  prose generator (`def *-cmds` or `def *-prose`). Runners are used in helper scripts;
  generators are called by skills to embed consistent command blocks into agent prompts.
- **No frontmatter in `_lib/` files** — they are function libraries, not skills. Claude Code
  does not discover them as skills; skills invoke them explicitly.
- **Non-breaking** — skill behaviour is identical. Only the source of the prose/commands
  changes. Conformance tests must still pass after every task.

## Out of Scope

- Converting any skill to a Nushell helper script (skills remain markdown).
- Merging `testing-philosophy` and `test-driven-development` (complementary, not redundant).
- Any Rust code changes.
- Updating `references/` or `helpers/` files beyond what's needed to fix broken links.

---

## Tasks

### Task 1: quality-gate.nu

**File**: `skills/_lib/quality-gate.nu`
**Run**: `cargo nextest run --workspace`

Write a Nushell module exporting:

````nushell
# Run nextest + clippy + fmt for a crate or workspace.
# Exits non-zero on any failure.
export def run-quality-gate [crate?: string] {
    let scope = if ($crate | is-empty) { ["--workspace"] } else { ["-p" $crate] }
    do { cargo nextest run ...$scope } | complete | if $in.exit_code != 0 { exit $in.exit_code }
    do { cargo clippy ...$scope -- -D warnings } | complete | if $in.exit_code != 0 { exit $in.exit_code }
    do { cargo fmt --all --check } | complete | if $in.exit_code != 0 { exit $in.exit_code }
}

# Return the canonical gate command block as a markdown code block string.
export def quality-gate-cmds [crate?: string]: string {
    let scope = if ($crate | is-empty) { "--workspace" } else { $"-p ($crate)" }
    $"```bash
cargo nextest run ($scope)
cargo clippy ($scope) -- -D warnings
cargo fmt --all --check
```"
}
````

Verify: `nu -c 'use skills/_lib/quality-gate.nu *; quality-gate-cmds' | str contains "nextest"`

### Task 2: guardrails.nu

**File**: `skills/_lib/guardrails.nu`
**Run**: `nu -c 'use skills/_lib/guardrails.nu *; branch-guard-cmds "main"'`

Write a Nushell module exporting:

```nushell
# Verify current branch matches expected. Exit 1 if not.
export def check-branch [expected: string] {
    let current = (git branch --show-current | str trim)
    if $current != $expected {
        print $"ERROR: expected branch ($expected), got ($current). Stopping."
        exit 1
    }
}

# Return branch-guard prose for embedding in agent prompts.
export def branch-guard-cmds [expected: string]: string {
    $"Run: git branch --show-current
   Verify output = ($expected). Stop immediately if not."
}

# Return the 3-attempt / BLOCKED rule prose.
export def blocked-rule []: string {
    "If stuck after 3 attempts on any item: write BLOCKED.md at the worktree root. Stop.
Do not retry with identical parameters."
}

# Return the never-no-verify rule prose.
export def no-verify-rule []: string {
    "Never use --no-verify on commits. Pre-commit hooks always run."
}
```

Verify: `nu -c 'use skills/_lib/guardrails.nu *; blocked-rule'` prints the rule text.

### Task 3: review-rules.nu

**File**: `skills/_lib/review-rules.nu`
**Run**: `nu -c 'use skills/_lib/review-rules.nu *; severity-table'`

```nushell
# Severity table as markdown.
export def severity-table []: string {
"| Level      | Action                          |
| ---------- | ------------------------------- |
| Blocking   | Must fix before merge           |
| Suggestion | Should fix; explain if skipping |
| Nitpick    | Optional; fix in one pass       |"
}

# One-pass rule prose.
export def one-pass-rule []: string {
    "Apply all severity levels in one pass. Do not commit after fixing only blocking
issues and leave suggestions for a follow-up — that creates noisy fix histories."
}

# False-positive handling protocol prose.
export def false-positive-protocol []: string {
    "When a reviewer (sentinel, clippy, obfsck) flags test data, string literals, or
fixture content:
- Add a per-site `#[allow(...)]` or allowlist entry immediately
- Do not change test content to work around the flag
- Document why the allowlist entry was added"
}
```

Verify: `nu -c 'use skills/_lib/review-rules.nu *; one-pass-rule'` returns the rule.

### Task 4: dispatch.nu

**File**: `skills/_lib/dispatch.nu`
**Run**: `nu -c 'use skills/_lib/dispatch.nu *; wave-state-init ["crate-a" "crate-b"]'`

```nushell
# Create a git worktree for an issue branch.
export def setup-worktree [repo_root: string, issue: int] {
    let path = $"($repo_root)/.worktrees/issue-($issue)"
    let branch = $"issue/($issue)"
    git -C $repo_root worktree add $path -b $branch
}

# Remove a worktree and delete its branch.
export def teardown-worktree [repo_root: string, issue: int] {
    let path = $"($repo_root)/.worktrees/issue-($issue)"
    let branch = $"issue/($issue)"
    git -C $repo_root worktree remove $path
    git -C $repo_root branch -d $branch
}

# Build a self-contained agent prompt for a worktree-based issue agent.
export def agent-prompt [repo_root: string, issue: int, title: string, body: string]: string {
    let wt = $"($repo_root)/.worktrees/issue-($issue)"
    let branch = $"issue/($issue)"
    $"You are implementing GitHub issue #($issue): ($title)

Worktree absolute path: ($wt)
Branch: ($branch)

Issue body:
($body)

Workflow:
1. Run: git -C ($wt) branch --show-current
   Verify output = ($branch). Stop immediately if not.
2. Read all files listed in the issue body before writing anything.
   Use absolute paths — do NOT use cd.
3. For each file to create/modify:
   a. Write a FAILING test if applicable.
      Run: cargo nextest run --workspace --manifest-path ($repo_root)/Cargo.toml
      Confirm FAIL.
   b. Implement minimum code to pass.
      Run: cargo nextest run --workspace --manifest-path ($repo_root)/Cargo.toml
      All green.
   c. Run: cargo clippy --workspace --manifest-path ($repo_root)/Cargo.toml -- -D warnings
      Zero warnings.
4. Commit:
   git -C ($wt) add -A
   git -C ($wt) commit -m \"feat: <summary> fixes #($issue)\"
5. Final check:
   cargo nextest run --workspace --manifest-path ($repo_root)/Cargo.toml
   cargo clippy --workspace --manifest-path ($repo_root)/Cargo.toml -- -D warnings
6. Report: files created, tests added, commit SHA, any blockers.

If stuck after 3 attempts: write BLOCKED.md at ($wt)/BLOCKED.md. Stop.
Do NOT modify files outside ($wt)."
}

# Emit initial wave-status JSON for a list of agent names.
export def wave-state-init [agents: list<string>]: string {
    let entries = ($agents | each { |a|
        $"    \"($a)\": {\"status\": \"pending\", \"branch\": \"\", \"commits\": []}"
    } | str join ",\n")
    $"{\n  \"wave\": 1,\n  \"agents\": {\n($entries)\n  }\n}"
}

# Merge a list of branches into main sequentially with --no-ff.
# Exits on first conflict or test failure.
export def integrate-branches [repo_root: string, branches: list<string>] {
    git -C $repo_root checkout main
    for branch in $branches {
        let result = (do { git -C $repo_root merge --no-ff $branch -m $"merge: ($branch)" } | complete)
        if $result.exit_code != 0 {
            print $"CONFLICT merging ($branch) — resolve manually before continuing."
            exit 1
        }
    }
    # Final suite
    do { cargo nextest run --workspace --manifest-path $"($repo_root)/Cargo.toml" } | complete
    | if $in.exit_code != 0 { exit $in.exit_code }
}
```

Verify: `nu -c 'use skills/_lib/dispatch.nu *; wave-state-init ["a" "b"]'` emits valid JSON.

### Task 5: Update cap, ci-fix, tdd, refactoring, verification-before-completion

**Files**:

- `skills/cap/SKILL.md`
- `skills/ci-fix/SKILL.md`
- `skills/test-driven-development/SKILL.md`
- `skills/refactoring/SKILL.md`
- `skills/verification-before-completion/SKILL.md`

For each: replace the inlined gate command block and branch-guard prose with a reference
to the `_lib` functions. Pattern:

````markdown
Run the quality gate:

```bash
nu skills/_lib/quality-gate.nu run-quality-gate [<crate>]
```
````

Or embed the commands directly (generated by `quality-gate-cmds`):

```bash
cargo nextest run --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --all --check
```

````

And replace branch-guard prose with:

```markdown
Verify branch before committing (`guardrails.nu check-branch <expected>`):

```bash
git branch --show-current   # must match expected branch — stop if not
````

````

After each edit: `just conformance` must still pass.

### Task 6: Update code-review and receiving-review

**Files**:
- `skills/code-review/SKILL.md`
- `skills/receiving-review/SKILL.md`

Replace the severity table, one-pass rule, and false-positive protocol with references
to `review-rules.nu`:

```markdown
Severity levels (from `skills/_lib/review-rules.nu severity-table`):

| Level      | Action                          |
| ---------- | ------------------------------- |
| Blocking   | Must fix before merge           |
| Suggestion | Should fix; explain if skipping |
| Nitpick    | Optional; fix in one pass       |

> One-pass rule: apply all severity levels in one pass before committing.
> See `review-rules.nu one-pass-rule` and `review-rules.nu false-positive-protocol`.
````

After edits: `just conformance` must pass.

### Task 7: Update parallel-agents and tackle-issues

**Files**:

- `skills/parallel-agents/SKILL.md`
- `skills/tackle-issues/SKILL.md`

In `parallel-agents`: replace Step 3 (wave state JSON), Step 4 (agent prompt template),
Step 5 (integration loop), and guardrails block with references to `dispatch.nu`:

````markdown
### Step 3: Initialize wave state

```bash
nu skills/_lib/dispatch.nu wave-state-init [<agent-names>] | save .ctx/wave-status.json
```
````

### Step 4: Build agent prompts

```bash
nu skills/_lib/dispatch.nu agent-prompt <repo_root> <issue> <title> <body>
```

### Step 5: Integrate results

```bash
nu skills/_lib/dispatch.nu integrate-branches <repo_root> [<branch-list>]
```

```

In `tackle-issues`: same replacement for Steps 3, 4, and 5.

After edits: `just conformance` must pass.

### Task 8: Run conformance and introspection

**Run**: `just conformance`

1. `just conformance` — all checks green.
2. `godmode:introspection` — run full audit; apply any findings in one pass.
3. Commit: `refactor(skills): extract _lib functions, deduplicate skill prose`

## Quality Rules

- No placeholders in any `.nu` file — all functions must be complete and runnable.
- Every `_lib/*.nu` function must be tested with a `nu -c` smoke check before the skill
  edits that reference it.
- `just conformance` must pass after every task.
- Skills must remain human-readable — references to `_lib` are additive annotations, not
  replacements for the prose explanation.
```
