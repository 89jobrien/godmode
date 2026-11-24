# Autonomous quality loop

Read this before running "improve quality" or "run the quality loop" against a workspace. It holds the four phases (MEASURE, FIX, VERIFY, COMMIT) with their commands and exit criteria. The finding codes and fix patterns the loop relies on are in SKILL.md and `references/fix-patterns.md`.

When asked to "improve quality" or "run the quality loop", execute this
phased workflow. Each phase has explicit scan scope and exit criteria.

<rustqual-phase name="MEASURE" mode="read-only">
Establish the baseline. Always run from the workspace root.

```bash
rustqual --init                                       # if no rustqual.toml
rustqual . --no-fail --save-baseline .ctx/rustqual-baseline.json
rustqual . --no-fail --format sarif > .ctx/rustqual-baseline.sarif.json
```

Record the score and finding count from the baseline file — do not
transcribe numbers into plan docs. The baseline file is the canonical
reference for all subsequent comparisons.

If per-crate breakdown is needed for a plan, run each crate individually
and note that per-crate totals will sum to less than the workspace total.
</rustqual-phase>

<rustqual-phase name="FIX" mode="read-write" max-iterations="10">
Iterative improvement loop. Each iteration:

1. Run `rustqual . --no-fail --compare .ctx/rustqual-baseline.json`
   to see current delta from baseline
2. Pick the single highest-impact finding category:
   - Count findings per category (DEAD_CODE, VIOLATION, TQ_UNTESTED, etc.)
   - Multiply count x category weight to rank impact
   - Prefer categories with real fixes over suppression-only categories
3. Fix up to 5 instances of that category
4. Run `taskit test run` — if tests fail, revert and try next category
5. Re-run `rustqual . --no-fail` — confirm score improved
6. If score >= target OR 10 iterations reached, transition to VERIFY

Print after each iteration:

| Iter | Category  | Files Changed     | Tests   | Score          |
| ---- | --------- | ----------------- | ------- | -------------- |
| 1    | DEAD_CODE | config.rs, oci.rs | 89 pass | 83.9% -> 98.6% |

</rustqual-phase>

<rustqual-phase name="VERIFY" mode="read-only">
Final comparison against the original baseline.

```bash
rustqual . --no-fail --compare .ctx/rustqual-baseline.json
```

Report the delta: starting score, ending score, findings removed.
Diff against `.ctx/rustqual-baseline.sarif.json` for per-finding detail.
If the score regressed, return to FIX.

Exit criteria (any of):

- Quality score exceeds target (default 95%)
- 10 iterations completed in FIX
- Only false-positive findings remain (cross-crate DEAD_CODE, cfg-gated
  code, CLI dispatch TQ_UNTESTED)
  </rustqual-phase>

<rustqual-phase name="COMMIT" mode="commit">
Update the baseline and commit.

```bash
rustqual . --no-fail --save-baseline .ctx/rustqual-baseline.json
```

Commit with message: `quality: raise rustqual score from X% to Y%`

Include both `rustqual.toml` and `.ctx/rustqual-baseline.json` in the
commit so the baseline tracks with the code that produced it.
</rustqual-phase>
