---
name: "godmode:mistake-tracker"
description: >
  Maintain a catalog of recurring mistakes, error patterns, and failure modes detected
  across sessions, commits, and CI runs. Use this skill to detect what keeps breaking,
  find root patterns, and capture prevention strategies.
requires: []
next: []
---

# Mistake Tracker

A persistent ledger at `.ctx/godmode/memory-bank/mistakes.md` that catalogs recurring error
patterns, failure modes, and process mistakes. The ledger is grounded in actual session
traces and git history — not assumptions.

## When to Use

- When asked "what mistakes keep happening?" or "why does this keep breaking?"
- After recurring CI failures or local test flakes
- Before a major release, to surface systemic issues
- When tracking prevents the same bug from shipping multiple times
- As a feedback loop to improve process and infrastructure

## Detection Categories

### 1. Clippy Lint Recurrence

Repeated clippy warnings with the same lint ID appearing across multiple sessions or
commits. Indicates a pattern the codebase is prone to that needs either:

- Systemic suppression (if false positive or accepted)
- Refactoring (if genuine pattern to eliminate)
- Compiler upgrade (if lint behavior changed)

Example:

```
Pattern: clippy::too_many_arguments
Occurrences: 5 (sessions 2026-05-10, 2026-05-12, 2026-05-15)
Affected: crates/godmode-core/src/session.rs
Prevention: Apply deny rule or refactor functions to use builder pattern
```

### 2. Test Failures

Same test failing across different branches or session runs. Indicates:

- Flaky test (race condition, timing dependency)
- Environment-specific failure (only fails on CI, only on macOS, etc.)
- Real bug that resurfaces on different code paths
- Test that needs isolation or setup fixes

Example:

```
Pattern: runnable_returns_tasks
Occurrences: 3 (2026-05-08, 2026-05-11, 2026-05-14)
Affected: godmode-core integration tests
Prevention: Add test-level retry or fix race condition in task graph locking
```

### 3. Process Errors

Repeated mistakes in git/branch/merge workflow:

- Committing to main instead of feature branch (subagent error)
- Force-pushing when merge was intended
- Stash drops or data loss
- Merging out-of-order, causing integration conflicts

Example:

```
Pattern: subagent commit to main
Occurrences: 2 (commits abc123d, def456g)
Prevention: Add git branch guard hook; require explicit confirmation for main commits
```

### 4. Hook False Positives

Same hook repeatedly blocking legitimate commits:

- Pre-commit obfsck flagging test fixture URLs as secrets
- Pre-commit pathspec pattern too broad, matching unintended files
- Hook regex matching variable names like `password` in test code

Example:

```
Pattern: obfsck false positive on localhost IPs in test file
Occurrences: 4 (added to allowlist 2026-05-01, 2026-05-06, 2026-05-12)
Prevention: Consolidate allowlist entries; add per-line allow comments to tests
```

### 5. Reverts

Commits that were reverted because they broke something. Indicates a mistake made it
past initial review and testing. Root cause may be:

- Test coverage gap
- Review process gap
- Integration testing gap
- Race condition or environment-specific issue

Example:

```
Pattern: revert: godmode task start breaks on empty graph
Occurrences: 1 (commit rev123, 2026-05-13)
Prevention: Add integration test for empty graph; add guard in start() function
```

## Ledger Format

The file `.ctx/godmode/memory-bank/mistakes.md` uses Markdown with frontmatter:

```yaml
---
version: 1
last_updated: 2026-06-02
next_review: 2026-06-09
---

# Recurring Mistakes Ledger

## Clippy Lints

### clippy::too_many_arguments
- **Occurrences**: 5
- **Dates**: 2026-05-10, 2026-05-12, 2026-05-15
- **Affected**: godmode-core/src/session.rs (Session::start_task)
- **Prevention**: Refactor functions to use builder pattern or struct wrapper
- **Notes**: Pattern appears when Session methods grow in scope

## Test Failures

### runnable_returns_tasks (flake)
- **Occurrences**: 3
- **Dates**: 2026-05-08, 2026-05-11, 2026-05-14
- **Affected**: godmode-core tests (task graph resolution)
- **Prevention**: Fix race condition in test setup or add retry logic
- **Notes**: Fails intermittently; may indicate test isolation issue

## Process Errors

### Subagent commits to main
- **Occurrences**: 2
- **Dates**: 2026-05-06 (abc123d), 2026-05-11 (def456g)
- **Prevention**: Pre-commit hook to guard against commits to main; require confirmation
- **Notes**: Subagents sometimes forget to switch branches before committing

## Hook False Positives

### obfsck: localhost IPs in test fixtures
- **Occurrences**: 4
- **Dates**: Allowlist entries 2026-05-01, 2026-05-06, 2026-05-12
- **Affected**: tests/fixtures/ (various)
- **Prevention**: Add per-file `allowlist:` directives in test files
- **Notes**: Overly broad IP pattern catches test localhost addresses

## Reverts

### revert: godmode task start breaks on empty graph
- **Occurrences**: 1
- **Dates**: 2026-05-13 (rev123)
- **Prevention**: Add integration test for empty graph edge case
- **Notes**: Review missed that empty graph returns error instead of empty list
```

## Integration with Other Skills

### godmode:verification-before-completion

Check the mistake ledger before shipping. Flag if:

- A pattern in the ledger affects the code being shipped
- Prevention strategies have not been applied
- The same mistake appears in new commits

### godmode:self-reflect

Reference the mistake ledger when reflecting on session outcomes. Use findings to
improve processes and identify where reviews or tests need strengthening.

### godmode:introspection

Include mistake ledger review in conformance checks. Verify:

- Ledger is current (updated within last 5 sessions)
- Prevention strategies are actionable
- Patterns have not been resolved but forgotten in the ledger

## Update Rules

- **After each session**: append any newly detected patterns
- **Weekly review**: check if prevention strategies have been implemented
- **When patterns are resolved**: add resolution note with date
- **Never delete entries**: keep full history for post-mortems and trend analysis
- **Consolidate duplicates**: if the same pattern appears under different names,
  merge entries and update counts

## Output Format

When reporting findings to the user, use:

```
## Recurring Mistakes Detected

### Clippy Lints (2 patterns)
- clippy::too_many_arguments (5 occurrences)
- clippy::needless_borrow (2 occurrences)

### Test Failures (1 pattern)
- runnable_returns_tasks (3 occurrences, flaky)

### Process Errors (1 pattern)
- subagent commits to main (2 occurrences)

### Hook False Positives (1 pattern)
- obfsck: localhost IPs (4 occurrences, allowlisted)

### Reverts (1 pattern)
- task start breaks on empty graph (1 occurrence)

**Next Steps**: Review `.ctx/godmode/memory-bank/mistakes.md` for prevention strategies and
see which can be implemented before the next release.
```

## Guardrails

- All patterns must be backed by actual traces or git history — never invent
- Occurrence counts must be exact; estimate only if source is unclear
- Dates must be accurate (use commit dates or .jsonl timestamps)
- Prevention strategies must be specific and actionable
- Do not modify source code; only catalog and recommend
- Keep the ledger append-only for audit trail purposes
