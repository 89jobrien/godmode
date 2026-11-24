# Layer 3: Adversarial Review

Read this before running the adversarial pass or checking claims against the hallucination patterns. The adversarial questions and red flags are in layer-3-adversarial-questions-and-red-flags.md.

Switch posture entirely. Assume the output contains errors and actively try to find them.

### Hallucination Pattern Checklist

Check every claim for these patterns:

1. **Fabricated citation** — specific case, paper, or statute that could not be found in
   Layer 2. Most dangerous pattern: looks authoritative, is invented.

2. **Precise number without source** — specific statistic (e.g., "78% of companies…") with
   no indicated origin. Models generate plausible statistics that are entirely fabricated.

3. **Confident specificity on uncertain topics** — exact dates, precise amounts, definitive
   attributions in areas where experts disagree or facts are genuinely unknown.

4. **Plausible-but-wrong association** — ruling attributed to wrong court, quote to wrong
   person, law's provision described incorrectly while the law's name is right.

5. **Temporal confusion** — something described as current that may be outdated; events
   described in wrong sequence.

6. **Overgeneralization** — stated as universally true when it applies only in specific
   jurisdictions, contexts, or time periods. Very common in legal and regulatory content.

7. **Missing qualifiers** — nuanced topic presented as settled when significant exceptions,
   limitations, or counterarguments exist.

The `--deep` matrix that scores each claim against these seven patterns is in
`references/layer-3-deep-matrix.md`.
