# Layer 3: Adversarial Review

Read this before running the adversarial pass, checking claims against the hallucination patterns, or deciding which red flags to escalate.

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

### Adversarial Questions

For each major claim that passed Layers 1 and 2:

- What would make this wrong?
- Is there a common misconception in this area the model might have absorbed?
- Would a domain expert object to how this is stated?
- Is this claim from after the training cutoff — could it be outdated?

### Red Flags to Escalate

Escalate these prominently — do not bury them in the table:

- A citation that cannot be found anywhere
- A statistic with no identifiable source
- A legal or regulatory claim contradicting what authoritative sources say
- A claim stated with high confidence that is actually disputed or uncertain
