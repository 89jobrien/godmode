# Review Triage Guide

## Classification

| Class          | Definition                                    | Action                                      |
| -------------- | --------------------------------------------- | ------------------------------------------- |
| Valid          | Reviewer is correct                           | Fix it                                      |
| False positive | Reviewer misread context or flagged test data | Add allowlist entry, document why           |
| Scope creep    | Valid concern but outside this PR             | Decline politely, open follow-up issue      |
| Disagreement   | You have a technically different view         | Document reasoning, ask reviewer to confirm |

## False Positive Handling

When a tool (sentinel, clippy, obfsck, gitleaks) flags:

- Variable named `password` in a test → `#[allow(clippy::...)]` at the site
- `localhost` URL in a fixture → add to obfsck allowlist, document why
- A GitHub URL in generated content → add to gitleaks allowlist

Do NOT change the test content to work around the flag.

## Fix Order

Always fix in severity order within one commit:

1. Blocking (correctness, panics, data loss)
2. Suggestions (architecture, naming, missing tests)
3. Nitpicks (style, formatting)

Never commit after blocking-only fixes. One review cycle = one fix commit.

## Disagreement Protocol

Do not silently skip. Document in the PR:

```
Disagree with [finding]: [your reasoning]. Leaving as-is unless reviewer confirms.
```

Wait for explicit acknowledgement before skipping.

## Sentinel-Specific

```
sentinel run → triage all → fix all in one commit → re-run sentinel → done
```

Sentinel runs apply ALL severity levels. Do not cherry-pick only blockers.
