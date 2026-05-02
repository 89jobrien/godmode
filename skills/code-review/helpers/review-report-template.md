# Code Review — <file or feature>

Reviewed: `<files or diff range>`
Date: YYYY-MM-DD

---

## Blocking

<!-- Must fix before merge. Format: [file:line] issue — why it matters -->

- (none)

## Suggestions

<!-- Should fix; explain if skipping. -->

- (none)

## Nitpicks

<!-- Optional; batch into one pass. -->

- (none)

---

## False Positives

<!-- Reviewer/tool flagged something that is intentional. Document the allowlist entry added. -->

- (none)

---

## Post-Fix Verification

```bash
cargo nextest run --workspace
cargo clippy --workspace -- -D warnings
```

- [ ] All tests pass
- [ ] Clippy clean
- [ ] `godmode:verification-before-completion` run
