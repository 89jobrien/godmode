# Claim Categories

| Category        | What to look for                    | Examples                             |
| --------------- | ----------------------------------- | ------------------------------------ |
| **Factual**     | How things are or were              | "Python was created in 1991"         |
| **Statistical** | Numbers, percentages, quantities    | "95% of enterprises use cloud"       |
| **Citation**    | Specific documents, cases, laws     | "Under Section 230 of the CDA..."    |
| **Entity**      | Claims about people, orgs, products | "OpenAI was founded by..."           |
| **Causal**      | X caused Y, X leads to Y            | "This vulnerability allows RCE"      |
| **Temporal**    | Dates, timelines, sequences         | "v2.0 was released before the patch" |

## Confidence Ratings

| Rating               | Meaning                                     | User action                  |
| -------------------- | ------------------------------------------- | ---------------------------- |
| **VERIFIED**         | Supporting source found and linked          | Spot-check if critical       |
| **PLAUSIBLE**        | Consistent with knowledge; no source        | Verify before relying        |
| **UNVERIFIED**       | No supporting or contradicting evidence     | Do not rely on               |
| **DISPUTED**         | Contradicting evidence from credible source | Review contradiction         |
| **FABRICATION RISK** | Matches hallucination pattern               | Assume wrong until confirmed |

## Hallucination Patterns (Layer 3)

1. Fabricated citation — unfindable case/paper/statute
2. Precise number without source — unsourced statistics
3. Confident specificity on uncertain topics
4. Plausible-but-wrong association — right name, wrong details
5. Temporal confusion — outdated info presented as current
6. Overgeneralization — universal claim for specific context
7. Missing qualifiers — settled when actually disputed
