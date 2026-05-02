# Review rules: severity table, one-pass rule, false-positive protocol prose generators.

# Severity table as markdown.
export def severity-table [] {
"| Level      | Action                          |
| ---------- | ------------------------------- |
| Blocking   | Must fix before merge           |
| Suggestion | Should fix; explain if skipping |
| Nitpick    | Optional; fix in one pass       |"
}

# One-pass rule prose.
export def one-pass-rule [] {
    "Apply all severity levels in one pass. Do not commit after fixing only blocking
issues and leave suggestions for a follow-up — that creates noisy fix histories."
}

# False-positive handling protocol prose.
export def false-positive-protocol [] {
    "When a reviewer (sentinel, clippy, obfsck) flags test data, string literals, or
fixture content:
- Add a per-site `#[allow(...)]` or allowlist entry immediately
- Do not change test content to work around the flag
- Document why the allowlist entry was added"
}
