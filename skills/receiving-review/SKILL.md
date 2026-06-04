---
name: "godmode:receiving-review"
description: >
  How to process incoming code review feedback. Use when receiving review comments —
  from sentinel, a human reviewer, or clippy — before implementing changes.
requires: []
next: [verification-before-completion]
---

# Receiving Code Review

## Core Rule

```
READ ALL FEEDBACK BEFORE FIXING ANYTHING
```

Fixing items one-by-one as you read leads to partial fixes, missed interactions, and
noisy multi-commit histories. Read everything, plan, then fix in one pass.

## Process

### 1. Triage all feedback

Read every comment. Classify each as:

| Class          | Definition                                        |
| -------------- | ------------------------------------------------- |
| Valid          | Fix it                                            |
| False positive | Reviewer misread context — document why and skip  |
| Scope creep    | Outside the PR — decline politely, open follow-up |
| Disagreement   | You have a different view — discuss before acting |

Do not immediately implement. Triage first.

### 2. Confirm green baseline before changes

```bash
cargo nextest run --workspace
```

If red before your changes, fix the pre-existing failures first and flag them separately.

> (`review-rules.nu one-pass-rule`)

### 3. Fix all valid items in one pass

- Blocking → Suggestion → Nitpick (in that order)
- Do not commit after blocking-only fixes and leave suggestions for later
- One review cycle = one fix commit

> (`review-rules.nu false-positive-protocol`)

### 4. Handle false positives immediately

When a reviewer flags test data, fixture content, or legitimate variable names:

- Add `#[allow(...)]` at the call site with a comment explaining why
- Add to allowlist/exclusion if it's a hook (obfsck, gitleaks)
- Do not change the flagged content to work around the reviewer

### 5. Handle disagreements

Do not silently skip feedback you disagree with. Instead:

1. Document your reasoning in the PR/review thread
2. Ask the reviewer to confirm or reconsider
3. Only skip after explicit acknowledgement

### 6. Verify after fixes

```bash
cargo fmt --all --check
cargo clippy --workspace -- -D warnings
cargo nextest run --workspace
```

Then use `godmode:verification-before-completion` before marking done.

## Sentinel-Specific Rules

When running sentinel, apply ALL severity levels in one pass. The workflow is:

```
sentinel run → triage all findings → fix all in one commit → re-run sentinel → done
```

Never commit after fixing only blocking issues. Suggestions and nitpicks go in the
same commit.

## What Not to Do

- Do not ask "should I fix this?" for clearly valid blocking issues — just fix them
- Do not expand scope while fixing review items (no "while I'm here" changes)
- Do not silently skip feedback — either fix it or document why you're not
- Do not commit partial fixes and plan to address the rest "in a follow-up"

## Additional Resources

- **`references/triage-guide.md`** — classification table, false positive handling, disagreement protocol
- **`helpers/fix-pass-template.md`** — track a full review cycle in one doc
