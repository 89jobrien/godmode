# Autonomous quality loop: VERIFY and COMMIT

Read this after the FIX phase, when verifying the quality score against the baseline or committing the baseline with the quality changes.

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
